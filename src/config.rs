use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub route: Vec<Route>,
    pub retry_strategy: RetryStrategyConfig,
    pub rate_limiting: Option<RateLimitingConfig>,
}

#[derive(Deserialize, Clone)]
pub enum StrategyType {
    FixedDelay,
    ExponentialBackoff,
    LinearBackoff,
    RandomDelay,
    IncrementalBackoff,
    FibonacciBackoff,
    GeometricBackoff,
    HarmonicBackoff,
    JitterBackoff,
}

#[derive(Deserialize, Clone)]
pub struct RateLimitingConfig {
    pub strategy: String,
    pub limit: u32,
    pub window_size_seconds: u64,
    pub refill_amount: Option<u32>,
    pub refill_interval_seconds: Option<u64>,
    pub capacity: Option<usize>,
    pub leak_rate_seconds: Option<u64>,
    pub granularity_seconds: Option<u64>,
}

#[derive(Deserialize, Clone)]
pub struct RetryStrategyConfig {
    pub strategy_type: StrategyType,
    pub max_delay_secs: u64,
    pub max_attempts: usize,
    pub initial_delay_millis: u64,
    pub timeout_secs: u64,
    pub increment_millis: Option<u64>,
    pub min_delay_millis: Option<u64>,
    pub multiplier: Option<f64>,
    pub increment_step_millis: Option<u64>,
    pub step_increment_millis: Option<u64>,
}

#[derive(Deserialize, Clone)]
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
    pub health_check_endpoints: Option<HashMap<String, String>>,
}

//The resource endpoint might return data like the following JSON, which your load balancer would need to parse: {
//  "cpu_usage_percent": 20.5,
// "memory_usage_percent": 55.3
//}
