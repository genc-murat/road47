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
        let mut statuses = HashMap::new();
        for (addr, endpoint) in health_check_endpoints {
            let is_healthy = self.client.get(endpoint).send().await.is_ok();
            statuses.insert(addr.clone(), is_healthy);
        }
        statuses
    }
}
