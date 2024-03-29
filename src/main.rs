mod balance;
mod config;
mod proxy;
use crate::balance::BalanceStrategy;
use env_logger;
use mobc::Pool;
use road47::cache::Cache;
use road47::config::RequestModificationRule;
use road47::config_manager::ConfigManager;
use road47::health_checker::HealthChecker;
use road47::rate_limiter::create_rate_limiter;
use road47::tcp_connection_manager::TcpConnectionManager;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config_manager = Arc::new(RwLock::new(ConfigManager::load("Config.toml").await?));

    let config = {
        let read_guard = config_manager.read().await;
        read_guard.get_config().await
    };

    let rate_limiter = Arc::new(create_rate_limiter(config.rate_limiting));

    let health_checker = Arc::new(HealthChecker::new());
    let health_statuses = Arc::new(Mutex::new(HashMap::<String, bool>::new()));

    let config_manager_clone: Arc<RwLock<ConfigManager>> = Arc::clone(&config_manager);

    tokio::spawn(async move {
        let read_guard = config_manager_clone.read().await;
        read_guard.run("Config.toml".to_string()).await;
    });

    for route in config.route {
        let timeout = Duration::from_secs(route.timeout_seconds);
        let manager = TcpConnectionManager::initialize_with(
            route.target_addrs.clone(),
            Arc::clone(&config_manager),
        );
        let pool = Arc::new(Pool::builder().build(manager));

        let listener = TcpListener::bind(&route.listen_addr).await?;
        info!("Listening on: {}", route.listen_addr);

        let target_addrs = Arc::new(Mutex::new(VecDeque::from(route.target_addrs)));
        let balance_strategy = BalanceStrategy::from_str(&route.balance_strategy);
        let connection_counts = Arc::new(Mutex::new(HashMap::new()));
        let request_limits = Arc::new(Mutex::new(HashMap::new()));
        let max_requests_per_target = route.max_requests_per_target;
        let resource_endpoints = route
            .resource_endpoints
            .as_ref()
            .map(|endpoints| Arc::new(Mutex::new(endpoints.clone())));

        let cache = Arc::new(Mutex::new(Cache::new(
            route.cache_ttl_seconds.unwrap_or_default(),
            route.cache_capacity.unwrap_or_default(),
        )));
        let cache_enabled_endpoints = route.cache_enabled_endpoints.clone();
        let target_weights = route.target_weights.clone();

        let request_modification_rules: Vec<RequestModificationRule> = route
            .request_modification_rules
            .clone()
            .unwrap_or_else(Vec::new);

        if let Some(health_check_endpoints) = &route.health_check_endpoints {
            let health_check_endpoints_arc = Arc::new(health_check_endpoints.clone());
            let health_checker_clone = Arc::clone(&health_checker);
            let health_statuses_clone = Arc::clone(&health_statuses);
            tokio::spawn(async move {
                let mut tick_interval = interval(Duration::from_secs(30));
                loop {
                    tick_interval.tick().await;
                    let mut statuses = health_checker_clone
                        .check_health(&health_check_endpoints_arc)
                        .await;
                    if statuses.is_empty() {
                        error!("Health check failed. Considering all services as down/up based on your policy.");
                        statuses = health_check_endpoints_arc
                            .iter()
                            .map(|(key, _)| (key.clone(), true))
                            .collect::<HashMap<String, bool>>();
                    }

                    let mut health = health_statuses_clone.lock().await;
                    *health = statuses;
                }
            });
        }

        tokio::spawn(proxy::accept_connections(
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
            target_weights,
            Some(health_statuses.clone()),
            Some(rate_limiter.clone()),
            Some(request_modification_rules),
        ));
    }

    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
    }
}
