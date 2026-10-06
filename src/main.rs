
use axum::body;
use axum::extract::State;
use futures_util::stream::StreamExt;
use mongodb::Collection;

use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::result;

use std::sync::{Arc};
use tokio::sync::{Mutex};
use std::env;

use dotenv::dotenv;
use mongodb::bson::{self, Document};
use bson::doc;

use mongodb::{Client, options::ClientOptions};

use web_scrapper::http_response::HttpResponse;
use web_scrapper::visited_site::{self, VisitedSite};

use axum::{
    extract::{Query, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    routing::get,
    Router,
};

use serde::{Deserialize, Serialize};

pub type SharedSet = Arc<Mutex<HashSet<String>>>;
pub type SharedDeque = Arc<Mutex<VecDeque<String>>>;

#[derive(Deserialize)]
struct PostPayload {
    url: String,
}

#[derive(Clone)]
struct AppState {
    visited_memory : SharedSet,
    visited_sites_collection: Collection<bson::Document>,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    dotenv().ok();
    let mongo_connection_url : String = env::var("MONGO_CONNECTION_URL")?;
    let mongo_db_name : String = env::var("MONGO_DB")?; 

    let options = ClientOptions::parse(mongo_connection_url).await?;  
    let client = Client::with_options(options)?;

    let db = client.database(&mongo_db_name);

    let state = AppState {
        visited_memory: Arc::new(Mutex::new(HashSet::<String>::new())),
        visited_sites_collection: db.collection::<bson::Document>("visited_sites")
    };

    let previously_visited_sites: Vec<VisitedSite> = get_saved_responses(&state.visited_sites_collection).await?;

    {
        for site in previously_visited_sites {
            state.visited_memory.lock().await.insert(site.url);
        }
    }
   
    let axum_app = Router::new()
        .route("/", post(start_crawling).get(get_all))
        .with_state(state);
    

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, axum_app).await.unwrap();

    
    Ok(())

}

async fn start_crawling(State(state): State<AppState> , Json(payload): Json<PostPayload>){
    println!("Started Crawling!");

    let visited_memory = state.visited_memory;
    let visited_collection = state.visited_sites_collection;

    let address = payload.url;

    let url_queue: SharedDeque = Arc::new(Mutex::new(VecDeque::<String>::new()));
        url_queue.lock().await.push_back(address.to_string());

    loop {
        let next: Option<String>;
        {
            next = url_queue.lock().await.pop_front();
        }

        let Some(url) = next else { continue; };

        let is_new = {
            let mut visited = visited_memory.lock().await;
            visited.insert(url.clone())
        };

        if is_new {

            let url_queue_clone = Arc::clone(&url_queue);
            let visited_sites_collection_clone = visited_collection.clone();

            tokio::spawn(async move { 

                let full_url = url.clone();

                let (url, path) = web_scrapper::crawl_async::split_into_domain_and_path(&url).await;

                println!("\n url + path = {}{}", url, path);

                let (addtional_links, http_response ) = web_scrapper::crawl_async::crawl_async(&url, &path).await.unwrap_or((Vec::<String>::new(), None));

                //println!("\n\n\n{:?}", addtional_links);

                {
                    let mut queue = url_queue_clone.lock().await;
                    for newLink in addtional_links {
                        queue.push_back(newLink);
                    }
                }

                let http_response = http_response.unwrap_or(
                    HttpResponse { status_code: (400), headers: (HashMap::<String,String>::new()), body: ("".to_string()) }
                );

                save_to_db(&visited_sites_collection_clone, &full_url, &http_response).await.unwrap();

            });
    
        }
        
    }

}

async fn get_all(State(state): State<AppState>) -> impl IntoResponse {

    let clone = state.visited_memory.lock().await.clone();
    
    let body = Json(serde_json::json!({
                "previous_urls": clone
        }));

    (axum::http::StatusCode::OK, body).into_response()
}

async fn save_to_db(collection: &mongodb::Collection<Document>, url: &str, res: &HttpResponse) -> Result<(), Box<dyn std::error::Error>>{
    let bson_file = doc!(
        "url": url,
        "status_code": res.status_code.to_string(),
        "body": res.body.clone(),
    );

    let result = collection.insert_one(bson_file).await?;

    println!("SAVED TO DB: {:?}", result);

    Ok(())
}

async fn get_saved_responses(
    collection: &mongodb::Collection<Document>
) -> Result<Vec<VisitedSite>, Box<dyn Error>> {
    let mut result = Vec::new();

    let mut cursor = collection.find(doc! {}).await?;


    while let Some(site_result) = cursor.next().await {

        let current = site_result?;

        let code_i32 = current.get_i32("status_code").unwrap_or(200);
        let status_code = code_i32 as u16;
        
        let url = current.get_str("url").unwrap_or("").to_string();
        let body = current.get_str("body").unwrap_or("").to_string();

        let new_site = VisitedSite { status_code, url, body };
        result.push(new_site);
    } 
    
    Ok(result)
}
