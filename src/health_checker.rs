use futures::future::join_all;
use reqwest::Client;
use std::collections::HashMap;

pub struct HealthChecker {
    client: Client,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn check_health(
        &self,
        health_check_endpoints: &HashMap<String, String>,
    ) -> HashMap<String, bool> {
        let mut check_futures = Vec::new();

        for (addr, endpoint) in health_check_endpoints {
            let client = self.client.clone();
            let future = async move {
                let is_healthy = client.get(endpoint).send().await.is_ok();
                (addr.clone(), is_healthy)
            };
            check_futures.push(future);
        }

        let results = join_all(check_futures).await;
        let mut statuses = HashMap::new();
        for (addr, is_healthy) in results {
            statuses.insert(addr, is_healthy);
        }

        statuses
    }
}
