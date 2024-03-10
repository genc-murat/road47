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
            let metadata = fs::metadata(&config_path).await;
            if let Ok(metadata) = metadata {
                if let Ok(modified) = metadata.modified() {
                    let last_modified_read = self.last_modified.read().await;
                    if modified > *last_modified_read {
                        drop(last_modified_read);
                        match fs::read_to_string(&config_path).await {
                            Ok(config_contents) => {
                                match toml::from_str::<Config>(&config_contents) {
                                    Ok(config) => {
                                        let mut config_lock = self.config.write().await;
                                        *config_lock = config;
                                        let mut last_modified_write =
                                            self.last_modified.write().await;
                                        *last_modified_write = modified;
                                        println!("Configuration has been successfully reloaded.");
                                    }
                                    Err(e) => eprintln!("Failed to parse config: {}", e),
                                }
                            }
                            Err(e) => eprintln!("Failed to read config file: {}", e),
                        }
                    }
                }
            }
        }
    }

    pub async fn get_config(&self) -> Config {
        let config_lock = self.config.read().await;
        (*config_lock).clone()
    }
}
