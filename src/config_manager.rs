use crate::config::Config;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::time::interval;

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

    pub async fn run(&mut self, config_path: String) {
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
                println!("Yapılandırma dosyası değişti, yeniden yükleniyor...");
                self.last_modified = modified;
                let config_contents = fs::read_to_string(&config_path)
                    .await
                    .expect("Failed to read config file");
                let config: Config =
                    toml::from_str(&config_contents).expect("Failed to parse config");
                let mut config_lock = self.config.write().unwrap();
                *config_lock = config;
                println!("Yapılandırma başarıyla yeniden yüklendi.");
            }
        }
    }
}
