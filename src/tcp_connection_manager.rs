use mobc::{async_trait, Manager};
use std::io;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time;
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
        let mut last_err = None;
        for address in &self.server_addresses {
            match time::timeout(self.timeout, TcpStream::connect(address)).await {
                Ok(Ok(stream)) => return Ok(stream),
                Ok(Err(e)) => last_err = Some(e),
                Err(_) => {
                    last_err = Some(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "Connection timed out",
                    ))
                }
            }
        }
        Err(last_err.unwrap_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to connect")))
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
    // Initialize with a list of server addresses and a timeout
    pub fn initialize_with(server_addresses: Vec<String>, timeout: Duration) -> Self {
        TcpConnectionManager {
            server_addresses,
            timeout,
        }
    }
}
