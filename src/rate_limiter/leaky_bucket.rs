use crate::rate_limiter::RateLimiter;
use std::collections::{HashMap, VecDeque};
use tokio::sync::Mutex; 
use std::time::{Duration, Instant};
use log::{warn, info}; 

pub struct LeakyBucketRateLimiter {
    requests: Mutex<HashMap<String, VecDeque<Instant>>>,
    capacity: usize,
    leak_rate: Duration,
}

impl LeakyBucketRateLimiter {
    pub fn new(capacity: usize, leak_rate: Duration) -> Self {
        LeakyBucketRateLimiter {
            requests: Mutex::new(HashMap::new()),
            capacity,
            leak_rate,
        }
    }

    pub async fn allow(&self, key: &str) -> bool {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();
        let queue = requests.entry(key.to_owned()).or_insert_with(VecDeque::new);

        while queue
            .front()
            .map_or(false, |&t| now.duration_since(t) > self.leak_rate)
        {
            queue.pop_front();
        }

        if queue.len() < self.capacity {
            queue.push_back(now);
            info!("Request allowed for key: {}", key);
            true
        } else {
            warn!("Request denied for key: {}", key);
            false
        }
    }
}
