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
    // Örnek olarak, her hedef için dinamik olarak hesaplanan bir limit değeri döndürün
    // Bu örnekte, basitlik adına, mevcut bağlantı sayısına bağlı basit bir mantık kullanacağız
    let current_connections = connection_counts.get(addr).unwrap_or(&0);
    if *current_connections > 100 {
        50 // Eğer mevcut bağlantı sayısı 100'den fazlaysa, limiti 50 olarak belirle
    } else {
        100 // Aksi takdirde, limit 100 olsun
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
        max_requests_per_target: usize,
        resource_endpoints: Arc<Mutex<Vec<String>>>,
        target_weights: Option<HashMap<String, usize>>,
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
            BalanceStrategy::WeightedRoundRobin => {
                let weights = target_weights.unwrap_or_else(|| {
                    addrs
                        .iter()
                        .map(|addr| (addr.clone(), 1))
                        .collect::<HashMap<String, usize>>()
                });
                let total_weight: usize = weights.values().sum();
                let mut rng = rand::thread_rng();
                let mut weight_point = rng.gen_range(0..total_weight);
                for (addr, weight) in weights.iter() {
                    if weight_point < *weight {
                        return Some(addr.clone());
                    }
                    weight_point -= *weight;
                }
                None
            }
            BalanceStrategy::DynamicRateLimiting => {
                let addrs = target_addrs.lock().await;
                let counts = connection_counts.lock().await;

                let eligible_addrs: Vec<_> = addrs
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
                    .collect();

                if eligible_addrs.is_empty() {
                    None
                } else {
                    let mut rng = rand::thread_rng();
                    let index = rng.gen_range(0..eligible_addrs.len());
                    eligible_addrs.get(index).cloned()
                }
            }
        }
    }
}
