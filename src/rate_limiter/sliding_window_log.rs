use super::RateLimiter;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct SlidingWindowLogRateLimiter {
    requests: Mutex<HashMap<String, VecDeque<Instant>>>,
    limit: u32,
    window: Duration,
}

impl SlidingWindowLogRateLimiter {
    pub fn new(limit: u32, window: Duration) -> Self {
        SlidingWindowLogRateLimiter {
            requests: Mutex::new(HashMap::new()),
            limit,
            window,
        }
    }

    fn cleanup(&self, key: &str) {
        let mut requests = self.requests.lock().unwrap();
        if let Some(times) = requests.get_mut(key) {
            let now = Instant::now();
            while times
                .front()
                .map_or(false, |&t| now.duration_since(t) > self.window)
            {
                times.pop_front();
            }
        }
    }
}

impl RateLimiter for SlidingWindowLogRateLimiter {
    fn allow(&self, key: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();

        let now = Instant::now();
        self.cleanup(key);

        let entry = requests.entry(key.to_owned()).or_insert_with(VecDeque::new);

        if (entry.len() as u32) < self.limit {
            entry.push_back(now);
            true
        } else {
            false
        }
    }
}
