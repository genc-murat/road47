use crate::rate_limiter::RateLimiter;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex; 
use log::{info, warn}; 

pub struct TokenBucketRateLimiter {
    tokens: Mutex<HashMap<String, (Instant, u32)>>,
    capacity: u32,
    refill_time: Duration,
    refill_amount: u32,
}

impl TokenBucketRateLimiter {
    /// Constructs a new `TokenBucketRateLimiter`.
    /// 
    /// # Arguments
    /// * `capacity` - The maximum capacity of tokens in the bucket.
    /// * `refill_time` - The duration in which tokens are refilled.
    /// * `refill_amount` - The amount of tokens added to the bucket when refilled.
    pub fn new(capacity: u32, refill_time: Duration, refill_amount: u32) -> Self {
        TokenBucketRateLimiter {
            tokens: Mutex::new(HashMap::new()),
            capacity,
            refill_time,
            refill_amount,
        }
    }

    /// Asynchronous method to check and update the rate limit for a given key.
    /// 
    /// # Arguments
    /// * `key` - The key to check the rate limit for.
    /// 
    /// # Returns
    /// `true` if the request is allowed, otherwise `false`.
    pub async fn allow(&self, key: &str) -> bool {
        let mut tokens = self.tokens.lock().await;

        let now = Instant::now();
        let (last_refill, current_tokens) = tokens
            .entry(key.to_string())
            .or_insert((now, self.capacity));

        if now.duration_since(*last_refill) >= self.refill_time {
            let cycles = now.duration_since(*last_refill).as_secs() / self.refill_time.as_secs();
            let tokens_to_add = std::cmp::min(self.refill_amount * cycles as u32, self.capacity);
            *current_tokens = std::cmp::min(*current_tokens + tokens_to_add, self.capacity);
            *last_refill = now;

            info!("Refilled tokens for key {}: Now {} tokens", key, *current_tokens);
        }

        if *current_tokens > 0 {
            *current_tokens -= 1;
            info!("Token decremented for key {}: Now {} tokens", key, *current_tokens);
            true
        } else {
            warn!("Request denied due to rate limiting for key {}", key);
            false
        }
    }
}

#[async_trait::async_trait]
impl RateLimiter for TokenBucketRateLimiter {
    async fn allow(&self, key: &str) -> bool {
        self.allow(key).await
    }
}
