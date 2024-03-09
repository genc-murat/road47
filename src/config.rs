use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Config {
    pub route: Vec<Route>,
    pub retry_strategy: RetryStrategyConfig,
}

#[derive(Deserialize)]
pub struct RetryStrategyConfig {
    pub max_delay_secs: u64,
    pub max_attempts: usize,
    pub initial_delay_millis: u64,
    pub timeout_secs: u64,
}

#[derive(Deserialize)]
pub struct Route {
    pub listen_addr: String,
    pub target_addrs: Vec<String>,
    pub target_weights: Option<HashMap<String, usize>>,
    pub resource_usage_api: Option<Vec<String>>,
    pub timeout_seconds: u64,
    pub balance_strategy: String,
    pub max_requests_per_target: Option<usize>,
    pub resource_endpoints: Option<Vec<String>>,
    pub cache_enabled_endpoints: Option<Vec<String>>,
    pub cache_ttl_seconds: Option<u64>,
    pub cache_capacity: Option<usize>,
}

//The resource endpoint might return data like the following JSON, which your load balancer would need to parse: {
//  "cpu_usage_percent": 20.5,
// "memory_usage_percent": 55.3
//}
