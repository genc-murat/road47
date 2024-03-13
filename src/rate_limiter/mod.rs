mod fixed_window;
mod leaky_bucket;
mod sliding_window_counter;
mod sliding_window_log;
mod token_bucket;

use crate::config::RateLimitingConfig;
use crate::rate_limiter::fixed_window::FixedWindowRateLimiter;
use crate::rate_limiter::leaky_bucket::LeakyBucketRateLimiter;
use crate::rate_limiter::sliding_window_counter::SlidingWindowCounterRateLimiter;
use crate::rate_limiter::sliding_window_log::SlidingWindowLogRateLimiter;
use crate::rate_limiter::token_bucket::TokenBucketRateLimiter;
use log::error;
use std::time::Duration;

pub trait RateLimiter: Send + Sync {
    fn allow(&self, key: &str) -> bool;
}

struct NoOpRateLimiter;

impl RateLimiter for NoOpRateLimiter {
    fn allow(&self, _key: &str) -> bool {
        true
    }
}

pub fn create_rate_limiter(
    config: Option<RateLimitingConfig>,
) -> Box<dyn RateLimiter + Send + Sync> {
    match config {
        Some(rate_limiting_config) => match rate_limiting_config.strategy.as_str() {
            "FixedWindow" => Box::new(FixedWindowRateLimiter::new(
                rate_limiting_config.limit,
                Duration::from_secs(rate_limiting_config.window_size_seconds),
            )),
            "SlidingWindow" => Box::new(SlidingWindowLogRateLimiter::new(
                rate_limiting_config.limit,
                Duration::from_secs(rate_limiting_config.window_size_seconds),
            )),
            "TokenBucket" => {
                let refill_amount = rate_limiting_config.refill_amount.unwrap_or_else(|| {
                    panic!("TokenBucket strategy requires a refill_amount in the configuration")
                });
                Box::new(TokenBucketRateLimiter::new(
                    rate_limiting_config.limit,
                    Duration::from_secs(rate_limiting_config.window_size_seconds),
                    refill_amount,
                ))
            }
            "LeakyBucket" => {
                let capacity = rate_limiting_config.capacity.unwrap_or_else(|| {
                    panic!("LeakyBucket strategy requires a capacity in the configuration")
                });
                let leak_rate_seconds = rate_limiting_config.leak_rate_seconds.unwrap_or_else(|| {
                        panic!("LeakyBucket strategy requires a leak_rate_seconds in the configuration")
                    });
                Box::new(LeakyBucketRateLimiter::new(
                    capacity,
                    Duration::from_secs(leak_rate_seconds),
                ))
            }
            "SlidingWindowCounter" => {
                let granularity_seconds = rate_limiting_config.granularity_seconds.unwrap_or_else(|| {
                    panic!("SlidingWindowCounter strategy requires a granularity_seconds in the configuration")
                });
                Box::new(SlidingWindowCounterRateLimiter::new(
                    rate_limiting_config.limit,
                    Duration::from_secs(rate_limiting_config.window_size_seconds),
                    Duration::from_secs(granularity_seconds),
                ))
            }
            _ => {
                let err_msg = format!(
                    "Unsupported rate limiting strategy: {}",
                    rate_limiting_config.strategy
                );
                error!("{}", &err_msg);
                Box::new(NoOpRateLimiter {})
            }
        },
        None => Box::new(NoOpRateLimiter {}),
    }
}
