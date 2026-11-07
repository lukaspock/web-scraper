use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{sleep_until, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    next_slot: Arc<Mutex<HashMap<String, Instant>>>,
    delay: Duration,
}

impl RateLimiter {
    pub fn new(delay: Duration) -> Self {
        Self {
            next_slot: Arc::new(Mutex::new(HashMap::new())),
            delay,
        }
    }

    pub async fn wait(&self, origin: &str) {
        let my_slot = {
            let mut map = self.next_slot.lock().await;
            let now = Instant::now();
            let slot = map.get(origin).copied().unwrap_or(now).max(now);
            map.insert(origin.to_string(), slot + self.delay); 
            slot
        }; 

        sleep_until(my_slot).await;
    }
}

/// Key under which requests to `url` share one rate limit: the last two labels of the host
/// (`de.wikipedia.org` and `en.wikipedia.org` both become `wikipedia.org`).
/// Simplification: multi-part suffixes like `co.uk` are not recognized.
pub fn rate_limit_key(url: &str) -> String {
    let Some(host) = url::Url::parse(url).ok().and_then(|u| u.host_str().map(str::to_string)) else {
        return url.to_string();
    };

    if host.parse::<std::net::IpAddr>().is_ok() {
        return host;
    }

    let labels: Vec<&str> = host.split('.').collect();
    if labels.len() <= 2 {
        return host;
    }

    labels[labels.len() - 2..].join(".")
}
