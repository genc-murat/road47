use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex; 

pub struct FixedWindowRateLimiter {
    requests: Mutex<HashMap<String, (u64, u32)>>,
    limit: u32,
    window_size: Duration,
}

impl FixedWindowRateLimiter {
    pub fn new(limit: u32, window_size: Duration) -> Self {
        FixedWindowRateLimiter {
            requests: Mutex::new(HashMap::new()),
            limit,
            window_size,
        }
    }

    pub async fn allow(&self, key: &str) -> bool {
        let mut requests = self.requests.lock().await;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let window_start = current_time - (current_time % self.window_size.as_secs());

        let entry = requests.entry(key.to_string()).or_insert((window_start, 0));
        if entry.0 < window_start {
            *entry = (window_start, 0);
        }

        if entry.1 < self.limit {
            entry.1 += 1;
            true
        } else {
            false
        }
    }
}
