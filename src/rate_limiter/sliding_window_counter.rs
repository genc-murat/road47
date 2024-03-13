use crate::rate_limiter::RateLimiter;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct TimeWindow {
    start: Instant,
    count: u32,
}

pub struct SlidingWindowCounterRateLimiter {
    windows: Mutex<HashMap<String, Vec<TimeWindow>>>,
    limit: u32,
    window_size: Duration,
    granularity: Duration,
}

impl SlidingWindowCounterRateLimiter {
    pub fn new(limit: u32, window_size: Duration, granularity: Duration) -> Self {
        assert!(
            window_size > granularity,
            "Window size must be greater than granularity."
        );
        SlidingWindowCounterRateLimiter {
            windows: Mutex::new(HashMap::new()),
            limit,
            window_size,
            granularity,
        }
    }
}

impl RateLimiter for SlidingWindowCounterRateLimiter {
    fn allow(&self, key: &str) -> bool {
        let mut windows = self.windows.lock().unwrap();

        let now = Instant::now();
        let windows_to_keep = now - self.window_size;

        let entry = windows.entry(key.to_owned()).or_insert_with(Vec::new);
        entry.retain(|window| window.start >= windows_to_keep);

        if let Some(current_window) = entry.last_mut() {
            if now.duration_since(current_window.start) > self.granularity {
                entry.push(TimeWindow {
                    start: now,
                    count: 1,
                });
            } else {
                if current_window.count < self.limit {
                    current_window.count += 1;
                } else {
                    return false;
                }
            }
        } else {
            entry.push(TimeWindow {
                start: now,
                count: 1,
            });
        }

        let total_count: u32 = entry.iter().map(|window| window.count).sum();
        total_count <= self.limit
    }
}
