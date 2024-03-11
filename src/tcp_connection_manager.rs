use crate::config_manager::ConfigManager;
use crate::retry::connect_with_retry;
use mobc::{async_trait, Manager};
use std::io;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::RwLock;

pub struct TcpConnectionManager {
    pub server_addresses: Vec<String>,
    pub config_manager: Arc<RwLock<ConfigManager>>,
}

impl TcpConnectionManager {
    pub fn initialize_with(
        server_addresses: Vec<String>,
        config_manager: Arc<RwLock<ConfigManager>>,
    ) -> Self {
        TcpConnectionManager {
            server_addresses,
            config_manager,
        }
    }
}

#[async_trait]
impl Manager for TcpConnectionManager {
    type Connection = TcpStream;
    type Error = io::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let config = {
            let lock = self.config_manager.read().await;
            lock.get_config().await
        };

        let retry_strategy_config = config.retry_strategy;
        connect_with_retry(&self.server_addresses, retry_strategy_config).await
    }

    async fn check(&self, mut conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        let buf = [0; 0];
        match conn.write(&buf).await {
            Ok(_) => Ok(conn),
            Err(e) => Err(e),
        }
    }
}
