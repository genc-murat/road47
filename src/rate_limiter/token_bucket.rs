use crate::rate_limiter::RateLimiter;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct TokenBucketRateLimiter {
    tokens: Mutex<HashMap<String, (Instant, u32)>>,
    capacity: u32,
    refill_time: Duration,
    refill_amount: u32,
}

impl TokenBucketRateLimiter {
    pub fn new(capacity: u32, refill_time: Duration, refill_amount: u32) -> Self {
        TokenBucketRateLimiter {
            tokens: Mutex::new(HashMap::new()),
            capacity,
            refill_time,
            refill_amount,
        }
    }
}

impl RateLimiter for TokenBucketRateLimiter {
    fn allow(&self, key: &str) -> bool {
        let mut tokens = self.tokens.lock().unwrap();
        let now = Instant::now();
        let (last_refill, current_tokens) = tokens
            .entry(key.to_string())
            .or_insert((now, self.capacity));

        if now.duration_since(*last_refill) >= self.refill_time {
            let cycles = now.duration_since(*last_refill).as_secs() / self.refill_time.as_secs();
            let tokens_to_add = std::cmp::min(self.refill_amount * cycles as u32, self.capacity);
            *current_tokens = std::cmp::min(*current_tokens + tokens_to_add, self.capacity);
            *last_refill = now;
        }

        if *current_tokens > 0 {
            *current_tokens -= 1;
            true
        } else {
            false
        }
    }
}
