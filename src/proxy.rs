use crate::balance::BalanceStrategy;
use log::{info, warn};
use mobc::Error as MobcError;
use mobc::Pool;
use road47::tcp_connection_manager::TcpConnectionManager;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::io::{self};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::{self, Duration};

pub async fn accept_connections(
    listener: TcpListener,
    pool: Arc<Pool<TcpConnectionManager>>,
    target_addrs: Arc<Mutex<VecDeque<String>>>,
    timeout: Duration,
    balance_strategy: BalanceStrategy,
    connection_counts: Arc<Mutex<HashMap<String, usize>>>,
    request_limits: Arc<Mutex<HashMap<String, usize>>>,
    max_requests_per_target: usize,
    resource_endpoints: Arc<Mutex<Vec<String>>>,
) -> io::Result<()> {
    while let Ok((incoming, _)) = listener.accept().await {
        let target_addrs_clone = target_addrs.clone();
        let timeout_clone = timeout;
        let connection_counts_clone = connection_counts.clone();
        let request_limits_clone = request_limits.clone();
        let resource_endpoints_clone = resource_endpoints.clone();

        let pool_clone = pool.clone();

        tokio::spawn(async move {
            if let Some(target_addr) = balance_strategy
                .select_target(
                    target_addrs_clone.clone(),
                    connection_counts_clone.clone(),
                    request_limits_clone.clone(),
                    max_requests_per_target,
                    resource_endpoints_clone,
                )
                .await
            {
                if let Err(e) = proxy_connection(
                    incoming,
                    &target_addr,
                    timeout_clone,
                    connection_counts_clone,
                    pool_clone,
                )
                .await
                {
                    warn!("Error proxying connection to {}: {:?}", target_addr, e);
                }
            } else {
                warn!("No target addresses available.");
            }
        });
    }

    Ok(())
}

async fn proxy_connection(
    mut incoming: TcpStream,
    target_addr: &str,
    timeout: Duration,
    connection_counts: Arc<Mutex<HashMap<String, usize>>>,
    pool: Arc<Pool<TcpConnectionManager>>,
) -> io::Result<()> {
    let target_stream_future = pool.get();
    let mut target = match time::timeout(timeout, target_stream_future).await {
        Ok(Ok(stream)) => {
            info!("Connection established to {}", target_addr);
            stream
        }
        Ok(Err(e)) => match e {
            MobcError::Timeout => {
                warn!("Connection to {} timed out", target_addr);
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Connection timed out",
                ));
            }
            _ => {
                warn!("Failed to connect to {}: {:?}", target_addr, e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to connect to {}: {:?}", target_addr, e),
                ));
            }
        },
        Err(_) => {
            warn!("Connection to {} timed out", target_addr);
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Connection timed out",
            ));
        }
    };

    {
        let mut counts = connection_counts.lock().await;
        *counts.entry(target_addr.to_string()).or_insert(0) += 1;
    }

    let (mut ri, mut wi) = incoming.split();
    let (mut ro, mut wo) = target.split();

    let client_to_server = tokio::io::copy(&mut ri, &mut wo);
    let server_to_client = tokio::io::copy(&mut ro, &mut wi);

    match tokio::try_join!(client_to_server, server_to_client) {
        Ok(_) => info!("Proxy completed successfully for {}", target_addr),
        Err(e) => warn!("Proxy operation failed for {}: {:?}", target_addr, e),
    }

    {
        let mut counts = connection_counts.lock().await;
        if let Some(count) = counts.get_mut(target_addr) {
            *count = count.saturating_sub(1);
        }
    }

    Ok(())
}
