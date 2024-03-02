use crate::config::load_config;
use crate::retry::connect_with_retry;
use mobc::{async_trait, Manager};
use std::io;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::Duration;

pub struct TcpConnectionManager {
    pub server_addresses: Vec<String>,
    pub timeout: Duration,
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
        )
        .await
    }

    async fn check(&self, mut conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        // Attempt to write a zero-byte array to the connection.
        // This operation is non-destructive for most protocols.
        let buf = [0; 0]; // A zero-length buffer.
        match conn.write(&buf).await {
            Ok(_) => Ok(conn), // The connection is writable.
            Err(e) => Err(e),  // The connection has likely gone bad.
        }
    }
}

impl TcpConnectionManager {
    pub fn initialize_with(server_addresses: Vec<String>, timeout: Duration) -> Self {
        TcpConnectionManager {
            server_addresses,
            timeout,
        }
    }
}
