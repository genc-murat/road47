use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub route: Vec<Route>,
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
}

//The resource endpoint might return data like the following JSON, which your load balancer would need to parse: {
//  "cpu_usage_percent": 20.5,
// "memory_usage_percent": 55.3
//}
