use super::RateLimiter;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;  
use log::{info, warn}; 

pub struct SlidingWindowLogRateLimiter {
    requests: Mutex<HashMap<String, VecDeque<Instant>>>,
    limit: u32,
    window: Duration,
}

impl SlidingWindowLogRateLimiter {
    /// Constructs a new `SlidingWindowLogRateLimiter`.
    /// 
    /// # Arguments
    /// * `limit` - The maximum number of allowed requests in the given time window.
    /// * `window` - The duration of the sliding window.
    pub fn new(limit: u32, window: Duration) -> Self {
        SlidingWindowLogRateLimiter {
            requests: Mutex::new(HashMap::new()),
            limit,
            window,
        }
    }

    /// Cleans up old timestamps that are outside of the allowable time window.
    /// 
    /// # Arguments
    /// * `key` - The key for which old requests should be cleaned up.
    async fn cleanup(&self, key: &str) {
        let mut requests = self.requests.lock().await;
        if let Some(times) = requests.get_mut(key) {
            let now = Instant::now();
            while times
                .front()
                .map_or(false, |&t| now.duration_since(t) > self.window)
            {
                times.pop_front();  // Remove old timestamps
            }
        }
    }
}

#[async_trait::async_trait]
impl RateLimiter for SlidingWindowLogRateLimiter {
    /// Checks if a request is allowed for the specified key and registers it if so.
    /// 
    /// # Arguments
    /// * `key` - The key to check and register the request for.
    /// 
    /// # Returns
    /// `true` if the request is allowed, otherwise `false`.
    async fn allow(&self, key: &str) -> bool {
        let mut requests = self.requests.lock().await;

        let now = Instant::now();
        self.cleanup(key).await;

        let entry = requests.entry(key.to_owned()).or_insert_with(VecDeque::new);

        if (entry.len() as u32) < self.limit {
            entry.push_back(now);
            info!("Request allowed for key: {}", key);  // Log the allowed request
            true
        } else {
            warn!("Request denied for key: {}", key);  // Log the denied request
            false
        }
    }
}
