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