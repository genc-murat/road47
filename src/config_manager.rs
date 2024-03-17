use crate::config::Config;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::interval;

#[derive(Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    last_modified: Arc<RwLock<SystemTime>>,
}

impl ConfigManager {
    pub async fn load(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = fs::metadata(config_path).await?;
        let last_modified = metadata.modified()?;
        let config_contents = fs::read_to_string(config_path).await?;
        let config: Config = toml::from_str(&config_contents)?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            last_modified: Arc::new(RwLock::new(last_modified)),
        })
    }

    pub async fn run(&self, config_path: String) {
        let mut interval = interval(Duration::from_secs(5));

        loop {
            interval.tick().await;
            let should_reload = {
                let last_modified_lock = self.last_modified.read().await;
                let metadata = fs::metadata(&config_path).await.ok();
                metadata.map_or(false, |m| m.modified().ok() > Some(*last_modified_lock))
            };

            if should_reload {
                if let Err(e) = self.reload_config(&config_path).await {
                    eprintln!("Failed to reload config: {}", e);
                }
            }
        }
    }

    async fn reload_config(&self, config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config_contents = fs::read_to_string(config_path).await?;
        let config: Config = toml::from_str(&config_contents)?;
        let metadata = fs::metadata(config_path).await?;
        let modified = metadata.modified()?;

        let mut config_lock = self.config.write().await;
        *config_lock = config;

        let mut last_modified_lock = self.last_modified.write().await;
        *last_modified_lock = modified;

        println!("Configuration has been successfully reloaded.");

        Ok(())
    }

    pub async fn get_config(&self) -> Config {
        self.config.read().await.clone()
    }
}
