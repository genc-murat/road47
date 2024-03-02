use float_ord::FloatOrd;
use rand::Rng;
use reqwest::Error;
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize, Clone, Debug)]
struct ResourceUsage {
    cpu_usage_percent: f32,
    memory_usage_percent: f32,
}

async fn fetch_resource_usage(endpoint: &str) -> Result<ResourceUsage, Error> {
    let client = reqwest::Client::new();
    let resp = client.get(endpoint).send().await?;
    resp.json::<ResourceUsage>().await
}

#[derive(Clone, Copy)]
pub enum BalanceStrategy {
    RoundRobin,
    Random,
    LeastConnections,
    RateLimiting,
    ResourceBased,
}

impl BalanceStrategy {
    pub fn from_str(strategy: &str) -> Self {
        match strategy {
            "random" => BalanceStrategy::Random,
            "leastconnections" => BalanceStrategy::LeastConnections,
            "ratelimiting" => BalanceStrategy::RateLimiting,
            "resourcebased" => BalanceStrategy::ResourceBased,
            _ => BalanceStrategy::RoundRobin,
        }
    }

    pub async fn select_target(
        &self,
        target_addrs: Arc<Mutex<VecDeque<String>>>,
        connection_counts: Arc<Mutex<HashMap<String, usize>>>,
        request_limits: Arc<Mutex<HashMap<String, usize>>>,
        max_requests_per_target: usize,
        resource_endpoints: Arc<Mutex<Vec<String>>>,
    ) -> Option<String> {
        let mut addrs = target_addrs.lock().await;
        let addrs_len = addrs.len();
        let mut counts = connection_counts.lock().await;
        let mut limits = request_limits.lock().await;

        if addrs_len == 0 {
            return None;
        }

        match *self {
            BalanceStrategy::RoundRobin => {
                if let Some(addr) = addrs.pop_front() {
                    addrs.push_back(addr.clone());
                    Some(addr)
                } else {
                    None
                }
            }
            BalanceStrategy::Random => {
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..addrs_len);
                addrs.get(index).cloned()
            }
            BalanceStrategy::LeastConnections => {
                let target = addrs
                    .iter()
                    .min_by_key(|addr| counts.get(*addr).unwrap_or(&usize::MAX))
                    .cloned();
                if let Some(ref addr) = target {
                    *counts.entry(addr.clone()).or_insert(0) += 1;
                }
                target
            }
            BalanceStrategy::RateLimiting => {
                for addr in addrs.iter() {
                    let count = limits.entry(addr.clone()).or_insert(0);
                    if *count < max_requests_per_target {
                        *count += 1; // İstek sayısını artır
                        return Some(addr.clone());
                    }
                }
                None
            }
            BalanceStrategy::ResourceBased => {
                let endpoints = resource_endpoints.lock().await;
                let mut scores = Vec::new();
                for (index, endpoint) in endpoints.iter().enumerate() {
                    if let Ok(usage) = fetch_resource_usage(endpoint).await {
                        // Simple scoring based on CPU and memory usage
                        let score = usage.cpu_usage_percent + usage.memory_usage_percent;
                        scores.push((index, FloatOrd(score)));
                    }
                }

                if let Some((index, _)) = scores.iter().min_by_key(|(_, score)| score) {
                    let addrs = target_addrs.lock().await;
                    addrs.get(*index).cloned()
                } else {
                    None
                }
            }
        }
    }
}
