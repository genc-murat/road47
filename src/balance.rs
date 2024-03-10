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
    WeightedRoundRobin,
    DynamicRateLimiting,
}
fn calculate_dynamic_limit(addr: &String, connection_counts: &HashMap<String, usize>) -> usize {
    let current_connections = connection_counts.get(addr).unwrap_or(&0);
    if *current_connections > 100 {
        50
    } else {
        100
    }
}
impl BalanceStrategy {
    pub fn from_str(strategy: &str) -> Self {
        match strategy {
            "random" => BalanceStrategy::Random,
            "leastconnections" => BalanceStrategy::LeastConnections,
            "ratelimiting" => BalanceStrategy::RateLimiting,
            "resourcebased" => BalanceStrategy::ResourceBased,
            "weightedroundrobin" => BalanceStrategy::WeightedRoundRobin,
            "dynamicratelimiting" => BalanceStrategy::DynamicRateLimiting,
            _ => BalanceStrategy::RoundRobin,
        }
    }

    pub async fn select_target(
        &self,
        target_addrs: Arc<Mutex<VecDeque<String>>>,
        connection_counts: Arc<Mutex<HashMap<String, usize>>>,
        request_limits: Arc<Mutex<HashMap<String, usize>>>,
        max_requests_per_target: Option<usize>,
        resource_endpoints: Option<Arc<Mutex<Vec<String>>>>,
        target_weights: Option<HashMap<String, usize>>,
        health_statuses: Option<Arc<Mutex<HashMap<String, bool>>>>,
    ) -> Option<String> {
        let filtered_addrs = {
            let lock = target_addrs.lock().await;
            if let Some(health_statuses) = &health_statuses {
                let health = health_statuses.lock().await;
                lock.iter()
                    .filter(|addr| *health.get(*addr).unwrap_or(&true)) // Default to true if not found
                    .cloned()
                    .collect::<VecDeque<_>>()
            } else {
                lock.clone()
            }
        };

        let addrs_len = filtered_addrs.len();
        if addrs_len == 0 {
            return None;
        }

        match *self {
            BalanceStrategy::RoundRobin => {
                let addr = filtered_addrs.front().cloned();
                addr
            }
            BalanceStrategy::Random => {
                let mut rng = rand::thread_rng();
                filtered_addrs.get(rng.gen_range(0..addrs_len)).cloned()
            }
            BalanceStrategy::LeastConnections => {
                let counts = connection_counts.lock().await;
                filtered_addrs
                    .iter()
                    .min_by_key(|addr| counts.get(*addr).unwrap_or(&usize::MAX))
                    .cloned()
            }
            BalanceStrategy::RateLimiting => {
                let limits = request_limits.lock().await;
                filtered_addrs
                    .iter()
                    .find(|addr| {
                        let count = limits.get(*addr).unwrap_or(&0);
                        if let Some(max_requests) = max_requests_per_target {
                            *count < max_requests
                        } else {
                            true
                        }
                    })
                    .cloned()
            }
            BalanceStrategy::ResourceBased => {
                if let Some(resource_endpoints) = &resource_endpoints {
                    let endpoints = resource_endpoints.lock().await;
                    let mut scores = Vec::new();
                    for (index, endpoint) in endpoints.iter().enumerate() {
                        if let Ok(usage) = fetch_resource_usage(endpoint).await {
                            let score = usage.cpu_usage_percent + usage.memory_usage_percent;
                            scores.push((index, FloatOrd(score)));
                        }
                    }

                    scores
                        .iter()
                        .min_by_key(|(_, score)| score)
                        .and_then(|(index, _)| filtered_addrs.get(*index).cloned())
                } else {
                    None
                }
            }
            BalanceStrategy::WeightedRoundRobin => {
                let total_weight: usize = if let Some(ref weights) = target_weights {
                    weights.values().sum()
                } else {
                    filtered_addrs.len()
                };

                let mut rng = rand::thread_rng();
                let mut weight_point = rng.gen_range(0..total_weight);

                for addr in filtered_addrs.iter() {
                    let weight = match &target_weights {
                        Some(weights) => *weights.get(addr).unwrap_or(&1),
                        None => 1,
                    };

                    if weight_point < weight {
                        return Some(addr.clone());
                    }
                    weight_point -= weight;
                }

                None
            }

            BalanceStrategy::DynamicRateLimiting => {
                let counts = connection_counts.lock().await;
                filtered_addrs
                    .iter()
                    .filter_map(|addr| {
                        let limit = calculate_dynamic_limit(addr, &counts);
                        let current_count = counts.get(addr).unwrap_or(&0);

                        if *current_count < limit {
                            Some(addr.clone())
                        } else {
                            None
                        }
                    })
                    .next()
            }
        }
    }
}
