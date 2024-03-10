use crate::config::Config;
use crate::retry::connect_with_retry;
use mobc::{async_trait, Manager};
use std::fs;
use std::io;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub struct TcpConnectionManager {
    pub server_addresses: Vec<String>,
}

fn load_config() -> Result<Config, io::Error> {
    let config_str = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_str)?;
    Ok(config)
}

#[async_trait]
impl Manager for TcpConnectionManager {
    type Connection = TcpStream;
    type Error = io::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let config = load_config().expect("Failed to load config");

        connect_with_retry(
            &self.server_addresses,
            config.retry_strategy.max_attempts,
            config.retry_strategy.max_delay_secs,
            config.retry_strategy.initial_delay_millis,
            config.retry_strategy.timeout_secs,
        )
        .await
    }

    async fn check(&self, mut conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        let buf = [0; 0];
        match conn.write(&buf).await {
            Ok(_) => Ok(conn),
            Err(e) => Err(e),
        }
    }
}

impl TcpConnectionManager {
    pub fn initialize_with(server_addresses: Vec<String>) -> Self {
        TcpConnectionManager { server_addresses }
    }
}
