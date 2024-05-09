use crate::rate_limiter::RateLimiter;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use log::{info, warn};

struct TimeWindow {
    start: Instant,
    count: u32,
}

pub struct SlidingWindowCounterRateLimiter {
    windows: Mutex<HashMap<String, VecDeque<TimeWindow>>>,
    limit: u32,
    window_size: Duration,
    granularity: Duration,
}

impl SlidingWindowCounterRateLimiter {
    pub fn new(limit: u32, window_size: Duration, granularity: Duration) -> Self {
        assert!(
            window_size > granularity,
            "Window size must be greater than granularity to maintain a meaningful sliding window."
        );
        SlidingWindowCounterRateLimiter {
            windows: Mutex::new(HashMap::new()),
            limit,
            window_size,
            granularity,
        }
    }

    pub async fn allow(&self, key: &str) -> bool {
        let mut windows = match self.windows.lock().await {
            Ok(w) => w,
            Err(e) => {
                warn!("Failed to acquire lock: {:?}", e);
                return false;
            }
        };

        let now = Instant::now();
        let windows_to_keep = now - self.window_size;

        let entry = windows.entry(key.to_owned()).or_insert_with(VecDeque::new);

        while entry.front().map_or(false, |w| w.start < windows_to_keep) {
            entry.pop_front();
        }

        if let Some(current_window) = entry.back_mut() {
            let time_since_last = now.duration_since(current_window.start);
            if time_since_last > self.granularity {
                entry.push_back(TimeWindow {
                    start: now,
                    count: 1,
                });
            } else {
                if current_window.count < self.limit {
                    current_window.count += 1;
                } else {
                    info!("Request rejected for key: {}", key);
                    return false;
                }
            }
        } else {
            entry.push_back(TimeWindow {
                start: now,
                count: 1,
            });
        }

        let total_count: u32 = entry.iter().map(|window| window.count).sum();
        info!("Request allowed for key: {}, Total count: {}", key, total_count);
        total_count <= self.limit
    }
}
