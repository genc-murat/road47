mod balance;
mod config;
mod proxy;
use crate::balance::BalanceStrategy;
use crate::config::Config;
use env_logger;
use log::{error, info};
use mobc::Pool;
use road47::cache::Cache;
use road47::tcp_connection_manager::TcpConnectionManager;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;
use std::{fs, sync::Arc};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config_contents = fs::read_to_string("Config.toml")?;
    let config: Config = toml::from_str(&config_contents)?;

    for route in config.route {
        let timeout = Duration::from_secs(route.timeout_seconds);
        let manager = TcpConnectionManager::initialize_with(route.target_addrs.clone(), timeout);
        let pool = Arc::new(Pool::builder().build(manager));

        let listener = TcpListener::bind(&route.listen_addr).await?;
        info!("Listening on: {}", route.listen_addr);

        let target_addrs = Arc::new(Mutex::new(VecDeque::from(route.target_addrs)));
        let balance_strategy = BalanceStrategy::from_str(&route.balance_strategy);
        let connection_counts = Arc::new(Mutex::new(HashMap::new()));
        let request_limits = Arc::new(Mutex::new(HashMap::new()));
        let max_requests_per_target = route.max_requests_per_target;
        let resource_endpoints = Arc::new(Mutex::new(route.resource_endpoints));

        let cache = Arc::new(Mutex::new(Cache::new(
            route.cache_ttl_seconds.unwrap_or_default(),
        )));
        let cache_enabled_endpoints = route.cache_enabled_endpoints.clone();

        // Spawn a new task for accepting connections
        tokio::spawn(async move {
            if let Err(e) = proxy::accept_connections(
                listener,
                pool,
                target_addrs,
                timeout,
                balance_strategy,
                connection_counts,
                request_limits,
                max_requests_per_target,
                resource_endpoints,
                cache,
                cache_enabled_endpoints,
            )
            .await
            {
                error!("Error processing connection: {:?}", e);
            }
        });
    }

    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
