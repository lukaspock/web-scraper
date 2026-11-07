use axum::extract::State;
use futures_util::TryStreamExt;
use futures_util::stream::StreamExt;
use mongodb::Collection;
use tower_http::cors::CorsLayer;
use web_scrapper::rate_limiter::{self, RateLimiter};
use web_scrapper::robots::RobotsCache;

use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;

use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinSet;

use bson::doc;
use dotenv::dotenv;
use mongodb::bson::{self, Document};

use mongodb::{Client, options::ClientOptions};

use web_scrapper::http_response::HttpResponse;
use web_scrapper::visited_site::VisitedSite;

use axum::{
    Router,
    extract::{Json, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    routing::post,
};

use serde::Deserialize;
use serde::Serialize;

pub type SharedSet = Arc<Mutex<HashSet<String>>>;
pub type SharedDeque = Arc<Mutex<VecDeque<String>>>;

#[derive(Deserialize)]
struct PostPayload {
    url: String,
}

#[derive(Deserialize)]
struct SearchQueryParams {
    text: String,
}

#[derive(Deserialize)]
struct GetByTextPayload {
    text: String,
}

#[derive(Clone)]
struct AppState {
    visited_memory: SharedSet,
    visited_sites_collection: Collection<bson::Document>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchResult {
    url: String,
    title: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let mongo_connection_url: String = env::var("MONGO_CONNECTION_URL")?;
    let mongo_db_name: String = env::var("MONGO_DB")?;

    let options = ClientOptions::parse(mongo_connection_url).await?;
    let client = Client::with_options(options)?;

    let db = client.database(&mongo_db_name);

    let state = AppState {
        visited_memory: Arc::new(Mutex::new(HashSet::<String>::new())),
        visited_sites_collection: db.collection::<bson::Document>("visited_sites"),
    };

    setup(&state.visited_sites_collection).await?;

    let previously_visited_sites: Vec<VisitedSite> =
        get_saved_responses(&state.visited_sites_collection).await?;
    {
        for site in previously_visited_sites {
            state.visited_memory.lock().await.insert(site.url);
        }
    }

    let axum_app = Router::new()
        .route("/", post(start_crawling).get(get_all))
        .route("/search", get(get_by_text))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, axum_app).await.unwrap();

    Ok(())
}

async fn start_crawling(State(state): State<AppState>, Json(payload): Json<PostPayload>) {
    println!("Started Crawling!");

    let client = reqwest::Client::builder()
        .user_agent(web_scrapper::user_agent::header_value())
        .build()
        .unwrap();
    let mut robots_cache = RobotsCache::new();

    let visited_memory = state.visited_memory;
    let visited_collection = state.visited_sites_collection;

    let raw_address = payload.url.trim().to_string();
    let with_scheme = if raw_address.contains("://") {
        raw_address
    } else {
        format!("https://{}", raw_address)
    };

    let address = match url::Url::parse(&with_scheme) {
        Ok(parsed) if parsed.host_str().is_some() => parsed.to_string(),
        _ => {
            println!("INVALID START URL: {}", with_scheme);
            return;
        }
    };

    println!("FIRST ADDRESS: {}", address);

    let url_queue: SharedDeque = Arc::new(Mutex::new(VecDeque::<String>::new()));
    url_queue.lock().await.push_back(address.to_string());

    let rate_limiter = RateLimiter::new(Duration::from_secs(1));

    let mut tasks = JoinSet::new();

    loop {
        let next: Option<String>;
        {
            next = url_queue.lock().await.pop_front();
        }

        let Some(url) = next else {
            if tasks.join_next().await.is_none() {
                break;
            }
            continue;
        };

        let is_new = {
            let mut visited = visited_memory.lock().await;
            visited.insert(url.clone())
        };

        if !is_new {
            println!("ALREADY VISITED: {}", url);
        }

        if is_new {
            if !robots_cache
                .is_allowed_url(&client, &url, &rate_limiter)
                .await
            {
                println!("DISSALLOWED: {} ", url);
                continue;
            }

            println!("ALLOWED: {}", url);

            let url_queue_clone = Arc::clone(&url_queue);
            let visited_sites_collection_clone = visited_collection.clone();

            let rate_limiter_clone = rate_limiter.clone();
            let client_clone = client.clone();

            tasks.spawn(async move {
                let full_url = url.clone();

                rate_limiter_clone
                    .wait(&rate_limiter::rate_limit_key(&url))
                    .await;

                let (addtional_links, http_response) =
                    match web_scrapper::crawl_async::crawl_async(&client_clone, &url).await {
                        Ok(result) => result,
                        Err(e) => {
                            println!("CRAWL ERROR for {}: {}", full_url, e);
                            (Vec::<String>::new(), None)
                        }
                    };

                {
                    let mut queue = url_queue_clone.lock().await;
                    for new_link in addtional_links {
                        queue.push_back(new_link);
                    }
                }

                let http_response = http_response.unwrap_or(HttpResponse {
                    status_code: (500),
                    headers: (HashMap::<String, String>::new()),
                    body: ("".to_string()),
                    filtered_body: ("".to_string()),
                });

                save_to_db(&visited_sites_collection_clone, &full_url, &http_response)
                    .await
                    .unwrap();
            });
        }
    }

    println!("Crawling Finished!")
}

async fn get_all(State(state): State<AppState>) -> impl IntoResponse {
    let clone = state.visited_memory.lock().await.clone();

    let body = Json(serde_json::json!({
            "previous_urls": clone
    }));

    (axum::http::StatusCode::OK, body).into_response()
}

async fn get_by_text(
    State(state): State<AppState>,
    query_params: Query<SearchQueryParams>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    let search_text: &str = &query_params.text;

    let cursor = state
        .visited_sites_collection
        .clone_with_type::<SearchResult>()
        .find(doc! { "$text": { "$search": search_text } })
        .projection(doc! { "url": 1, "title": 1, "_id": 0 })
        .limit(20)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let results: Vec<SearchResult> = cursor
        .try_collect()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(results))
}

async fn save_to_db(
    collection: &mongodb::Collection<Document>,
    url: &str,
    res: &HttpResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    let bson_file = doc!(
        "url": url,
        "status_code": res.status_code.to_string(),
        "body": res.body.clone(),
        "filtered_body": res.filtered_body.clone(),
    );

    let result = collection.insert_one(bson_file).await?;

    println!("SAVED TO DB: {:?}", result);

    Ok(())
}

async fn setup(collection: &Collection<Document>) -> mongodb::error::Result<()> {
    let index = mongodb::IndexModel::builder()
        .keys(doc! { "filtered_body": "text" })
        .build();
    collection.create_index(index).await?;
    Ok(())
}

async fn get_saved_responses(
    collection: &mongodb::Collection<Document>,
) -> Result<Vec<VisitedSite>, Box<dyn Error>> {
    let mut result = Vec::new();

    let mut cursor = collection.find(doc! {}).await?;

    while let Some(site_result) = cursor.next().await {
        let current = site_result?;

        let code_i32 = current.get_i32("status_code").unwrap_or(200);
        let status_code = code_i32 as u16;

        let url = current.get_str("url").unwrap_or("").to_string();
        let body = current.get_str("body").unwrap_or("").to_string();

        let new_site = VisitedSite {
            status_code,
            url,
            body,
        };
        result.push(new_site);
    }

    Ok(result)
}
