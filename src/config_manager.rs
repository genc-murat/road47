use crate::config::Config;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::interval;

#[derive(Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    last_modified: SystemTime,
}

impl ConfigManager {
    pub async fn new(config_path: &str) -> Self {
        let metadata = fs::metadata(config_path)
            .await
            .expect("Failed to read file metadata");
        let last_modified = metadata
            .modified()
            .expect("Failed to get modification time");
        let config_contents = fs::read_to_string(config_path)
            .await
            .expect("Failed to read config file");
        let config: Config = toml::from_str(&config_contents).expect("Failed to parse config");

        ConfigManager {
            config: Arc::new(RwLock::new(config)),
            last_modified,
        }
    }

    pub async fn load(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = fs::metadata(config_path).await?;
        let last_modified = metadata.modified()?;
        let config_contents = fs::read_to_string(config_path).await?;
        let config: Config = toml::from_str(&config_contents)?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            last_modified,
        })
    }

    pub async fn run(&self, config_path: String) {
        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let metadata = match fs::metadata(&config_path).await {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            let modified = match metadata.modified() {
                Ok(time) => time,
                Err(_) => continue,
            };
            if modified > self.last_modified {
                println!("The configuration file has changed, reloading...");
                let config_contents = fs::read_to_string(&config_path)
                    .await
                    .expect("Failed to read config file");
                let config: Config =
                    toml::from_str(&config_contents).expect("Failed to parse config");

                let mut config_lock = self.config.write().await;
                *config_lock = config;
                println!("The configuration has been successfully reloaded.");
            }
        }
    }

    pub async fn get_config(&self) -> Config {
        let config_lock = self.config.read().await;
        (*config_lock).clone()
    }
}
