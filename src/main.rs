
use futures_util::stream::StreamExt;

use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::result;
use std::sync::{Arc, Mutex};
use std::env;

use dotenv::dotenv;
use mongodb::bson::{self, Document};
use bson::doc;
use mongodb::change_stream::event::ChangeNamespaceType::Collection;
use mongodb::{Client, options::ClientOptions};

use web_scrapper::http_response::HttpResponse;
use web_scrapper::visited_site::{self, VisitedSite};
pub type SharedSet = Arc<Mutex<HashSet<String>>>;
pub type SharedDeque = Arc<Mutex<VecDeque<String>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    dotenv().ok();
    let mongo_connection_url : String = dotenv::var("MONGO_CONNECTION_URL")?;
    let mongo_db_name : String = dotenv::var("MONGO_DB")?;

    let options = ClientOptions::parse(mongo_connection_url).await?;  
    let client = Client::with_options(options)?;

    let db = client.database(&mongo_db_name);

    let visited_sites_collection = db.collection::<bson::Document>("visited_sites");

    
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);

    let address = if args.len() > 1 {
            args[1].clone()
        } else {
            String::from("quotes.toscrape.com:443")
        };

    let visited_memory : SharedSet = Arc::new(Mutex::new(HashSet::<String>::new()));
    let url_queue: SharedDeque = Arc::new(Mutex::new(VecDeque::<String>::new()));

    let previously_visited_sites: Vec<VisitedSite> = get_saved_responses(&visited_sites_collection).await?;

    println!("previously_visited_sites: \n");

    {
        for site in previously_visited_sites {
            //println!("{}\n", site.url);
            visited_memory.lock().unwrap().insert(site.url);
        }
    }
    
    url_queue.lock().unwrap().push_back(address.to_string());

    
    loop {
        let next: Option<String>;
        {
            next = url_queue.lock().unwrap().pop_front();
        }

        let Some(url) = next else { continue; };

        let is_new = {
            let mut visited = visited_memory.lock().unwrap();
            visited.insert(url.clone())
        };

        if is_new {

            let url_queue_clone = Arc::clone(&url_queue);
            let visited_sites_collection_clone = visited_sites_collection.clone();

            tokio::spawn(async move { 

                let full_url = url.clone();

                let (url, path) = web_scrapper::crawl_async::split_into_domain_and_path(&url).await;

                println!("\n url + path = {}{}", url, path);

                let (addtional_links, http_response ) = web_scrapper::crawl_async::crawl_async(&url, &path).await.unwrap_or((Vec::<String>::new(), None));

                //println!("\n\n\n{:?}", addtional_links);

                {
                    let mut queue = url_queue_clone.lock().unwrap();
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

    println!("\n\nALL VISITED SITES: \n{:?}\n\n", visited_memory);

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
