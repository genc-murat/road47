use serde::Deserialize;
use std::fs;
use std::io;

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
}

#[derive(Deserialize)]
pub struct Route {
    pub listen_addr: String,
    pub target_addrs: Vec<String>,
    pub resource_usage_api: Vec<String>,
    pub timeout_seconds: u64,
    pub balance_strategy: String,
    pub max_requests_per_target: usize,
    pub resource_endpoints: Vec<String>,
    pub cache_enabled_endpoints: Vec<String>,
    pub cache_ttl_seconds: Option<u64>,
}

pub fn load_config() -> Result<Config, io::Error> {
    let config_str = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}

//The resource endpoint might return data like the following JSON, which your load balancer would need to parse: {
//  "cpu_usage_percent": 20.5,
// "memory_usage_percent": 55.3
//}
