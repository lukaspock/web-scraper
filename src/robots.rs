use std::collections::HashMap;
use robotstxt::DefaultMatcher;

use crate::rate_limiter;


#[derive(Default)]
pub struct RobotsCache {
   pub cache: HashMap<String, String>
}

impl RobotsCache {

    pub fn new() -> Self{
        Self{cache: HashMap::<String,String>::new()}
    }

    pub async fn is_allowed_url(&mut self, client: &reqwest::Client, url: &str, limiter: &rate_limiter::RateLimiter) -> bool {

        println!("IS ALLOWED URL CALLED WITH: {}", url);
        let Ok(parsed_url) = url::Url::parse(url) else { return false; };
        println!("{:?}", parsed_url);
        let origin = format!("{}://{}", parsed_url.scheme(), parsed_url.host_str().unwrap_or_default());

        if(!self.cache.contains_key(&origin)){

            limiter.wait(&origin).await;

            let robots_url = format!("{}/robots.txt", origin);
            let body = match client.get(&robots_url).send().await {
                Ok(res) if res.status().is_success() => {
                    let text = res.text().await.unwrap_or_default();
                    sanitize_robots_body(&robots_url, text)
                }
                _ => {String::new()}, // keine robots.txt = alles erlaubt
            };

            self.cache.insert(origin.clone(), body);
        }

        

        let robots_body = &self.cache[&origin];

        if robots_body.is_empty() {
            println!("RETURNS TRUE");
                return true;
        }

        // the robotstxt parser can panic on odd input, so a broken robots.txt must not kill the task
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut matcher = DefaultMatcher::default();
            matcher.one_agent_allowed_by_robots(robots_body, crate::user_agent::ROBOTS_TOKEN, url)
        }));

        match result {
            Ok(allowed) => allowed,
            Err(_) => {
                println!("ROBOTS PARSER FAILED for {}, treating as disallowed", url);
                false
            }
        }
    }
}

/// Max size crawlers have to honor (500 KiB, as in Google's robots.txt spec).
const MAX_ROBOTS_BYTES: usize = 500 * 1024;

/// Some servers answer /robots.txt with 200 and an HTML page (e.g. a soft 404 or a redirect to the
/// home page). That is not a robots.txt, so it is treated as "no rules".
fn sanitize_robots_body(robots_url: &str, mut body: String) -> String {
    if body.trim_start().starts_with('<') {
        println!("NOT A ROBOTS.TXT (looks like HTML): {}", robots_url);
        return String::new();
    }

    if body.len() > MAX_ROBOTS_BYTES {
        let mut end = MAX_ROBOTS_BYTES;
        while !body.is_char_boundary(end) {
            end -= 1;
        }
        body.truncate(end);
    }

    body
}