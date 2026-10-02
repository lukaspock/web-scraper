

use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};

pub type sharedSet = Arc<Mutex<HashSet<String>>>;
pub type sharedDeque = Arc<Mutex<VecDeque<String>>>;

#[tokio::main]
async fn main(){

    let address = "quotes.toscrape.com:443";

    let visited_memory : sharedSet = Arc::new(Mutex::new(HashSet::<String>::new()));
    let url_queue: sharedDeque = Arc::new(Mutex::new(VecDeque::<String>::new()));

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

            tokio::spawn(async move { 
                let (url, path) = web_scrapper::crawl_async::split_into_domain_and_path(&url).await;

                println!("\n url + path = {}{}", url, path);

                let addtional_links: Vec<String> = web_scrapper::crawl_async::crawl_async(&url, &path).await.unwrap_or(Vec::<String>::new());

                //println!("\n\n\n{:?}", addtional_links);

                {
                    let mut queue = url_queue_clone.lock().unwrap();
                    for newLink in addtional_links {
                        queue.push_back(newLink);
                    }
                }
            });
    
        }
    }

    println!("\n\nALL VISITED SITES: \n{:?}\n\n", visited_memory);

}