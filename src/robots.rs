use std::collections::HashMap;
use robotstxt::DefaultMatcher;

#[derive(Default)]
pub struct RobotsCache {
   pub cache: HashMap<String, String>
}

impl RobotsCache {

    pub fn new() -> Self{
        Self{cache: HashMap::<String,String>::new()}
    }

    pub async fn is_allowed_url(&mut self, client: &reqwest::Client, url: &str) -> bool {

        println!("IS ALLOWED URL CALLED WITH: {}", url);
        let parsed_url = url::Url::parse(url).unwrap();
        println!("{:?}", parsed_url);
        let origin = format!("{}://{}", parsed_url.scheme(), parsed_url.host_str().unwrap());

        if(!self.cache.contains_key(&origin)){
            let robots_url = format!("{}/robots.txt", origin);
            let body = match client.get(&robots_url).send().await {
                Ok(res) if res.status().is_success() => res.text().await.unwrap_or_default(),
                _ => {String::new()}, // keine robots.txt = alles erlaubt
            };

            self.cache.insert(origin.clone(), body);
        }

        

        let robots_body = &self.cache[&origin];

        println!("Body : {}", robots_body);

        if robots_body.is_empty() {
            println!("RETURNS TRUE");
                return true;
        }

        let mut matcher = DefaultMatcher::default();

        let alowed = matcher.one_agent_allowed_by_robots(robots_body, "MeinCrawler", url);

        alowed
        
    }
}