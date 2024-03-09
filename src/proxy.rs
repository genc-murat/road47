use crate::balance::BalanceStrategy;
use log::{info, warn};
use mobc::Error as MobcError;
use mobc::Pool;
use road47::cache::Cache;
use road47::tcp_connection_manager::TcpConnectionManager;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::net::tcp::{ReadHalf, WriteHalf};
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
    max_requests_per_target: Option<usize>,
    resource_endpoints: Option<Arc<Mutex<Vec<String>>>>,
    cache: Arc<Mutex<Cache>>,
    cache_enabled_endpoints: Option<Vec<String>>,
    target_weights: Option<HashMap<String, usize>>,
) -> io::Result<()> {
    while let Ok((incoming, _)) = listener.accept().await {
        let target_addrs_clone = target_addrs.clone();
        let timeout_clone = timeout;
        let connection_counts_clone = connection_counts.clone();
        let request_limits_clone = request_limits.clone();
        let resource_endpoints_clone = resource_endpoints.clone();

        let pool_clone = pool.clone();

        let cache_clone = cache.clone(); // Clone the cache for the spawned task
        let cache_enabled_endpoints_clone = cache_enabled_endpoints.clone();
        let target_weights_clone = target_weights.clone();
        tokio::spawn(async move {
            if let Some(target_addr) = balance_strategy
                .select_target(
                    target_addrs_clone.clone(),
                    connection_counts_clone.clone(),
                    request_limits_clone.clone(),
                    max_requests_per_target,
                    resource_endpoints_clone,
                    target_weights_clone,
                )
                .await
            {
                if let Err(e) = proxy_connection(
                    incoming,
                    &target_addr,
                    timeout_clone,
                    connection_counts_clone,
                    pool_clone,
                    cache_clone,
                    cache_enabled_endpoints_clone,
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
    cache: Arc<Mutex<Cache>>, // Cache is now correctly wrapped in an async Mutex
    cache_enabled_endpoints: Option<Vec<String>>,
) -> io::Result<()> {
    let requested_endpoint = extract_endpoint_from_stream(&mut incoming).await?;

    if let Some(ref cache_enabled_endpoints) = cache_enabled_endpoints {
        if cache_enabled_endpoints.contains(&requested_endpoint) {
            // Lock the cache asynchronously to ensure safe concurrent access
            let cache_lock = cache.lock().await; // Artık doğru şekilde asenkron bir kilit kullanılıyor
            if let Some(cached_data) = cache_lock.get(&requested_endpoint).await {
                // .await eklenmeli
                // Cached data found, send response
                return send_cached_response(incoming, &cached_data).await;
            }
        }
    }

    // No cached data found or caching not enabled for this endpoint, proceed with normal flow
    let target = connect_to_target(&pool, timeout, target_addr).await?;

    let result = proxy_traffic_and_cache_response(
        incoming,
        target,
        target_addr,
        &connection_counts,
        cache.clone(), // Pass cache clone to the function
        requested_endpoint.clone(),
        &cache_enabled_endpoints,
    )
    .await;

    result
}

async fn extract_endpoint_from_stream(stream: &mut TcpStream) -> io::Result<String> {
    let mut reader = BufReader::new(stream);
    let mut first_line = String::new();

    // Read the first line from the request
    let bytes_read = reader.read_line(&mut first_line).await?;

    // If no bytes were read, return an error
    if bytes_read == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "EOF reached before completing read",
        ));
    }

    // Example assumes the request is a well-formed HTTP GET request and extracts the path
    // HTTP GET request format: GET /path HTTP/1.1
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid HTTP request line",
        ));
    }

    // Assuming the request follows the standard format, the path is the second part
    let path = parts[1].to_string();

    Ok(path)
}

async fn send_cached_response(mut stream: TcpStream, cached_data: &[u8]) -> io::Result<()> {
    stream.write_all(cached_data).await?;
    stream.flush().await?;
    Ok(())
}

async fn connect_to_target(
    pool: &Arc<Pool<TcpConnectionManager>>,
    timeout: Duration,
    target_addr: &str,
) -> io::Result<TcpStream> {
    let target_stream_future = pool.get();
    match time::timeout(timeout, target_stream_future).await {
        Ok(Ok(connection)) => {
            info!("Connection established to {}", target_addr);
            let tcp_stream = connection.into_inner();
            Ok(tcp_stream)
        }
        Ok(Err(e)) => match e {
            MobcError::Timeout => {
                warn!("Connection to {} timed out", target_addr);
                Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Connection timed out",
                ))
            }
            _ => {
                warn!("Failed to connect to {}: {:?}", target_addr, e);
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to connect to {}: {:?}", target_addr, e),
                ))
            }
        },
        Err(_) => {
            warn!("Connection attempt to {} timed out", target_addr);
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Connection attempt timed out",
            ))
        }
    }
}

async fn proxy_traffic_and_cache_response(
    mut incoming: TcpStream,
    mut target: TcpStream,
    target_addr: &str,
    connection_counts: &Arc<Mutex<HashMap<String, usize>>>,
    cache: Arc<Mutex<Cache>>,
    requested_endpoint: String,
    cache_enabled_endpoints: &Option<Vec<String>>,
) -> io::Result<()> {
    // Increment connection count
    {
        let mut counts = connection_counts.lock().await;
        *counts.entry(target_addr.to_string()).or_insert(0) += 1;
    }

    let (mut ri, mut wi) = incoming.split();
    let (mut ro, mut wo) = target.split();
    match tokio::try_join!(
        tokio::io::copy(&mut ri, &mut wo),
        tokio::io::copy(&mut ro, &mut wi)
    ) {
        Ok(_) => info!("Proxy completed successfully for {}", target_addr),
        Err(e) => warn!("Proxy operation failed for {}: {:?}", target_addr, e),
    }

    let mut target_response_buffer = Vec::new();

    let proxy_result = tokio::try_join!(
        tokio::io::copy(&mut ri, &mut wo),
        read_and_write(&mut ro, &mut wi, &mut target_response_buffer),
    );

    if let Ok(_) = proxy_result {
        if cache_enabled_endpoints
            .as_ref()
            .map_or(false, |eps| eps.contains(&requested_endpoint))
        {
            let cache_lock = cache.lock().await;
            // Await the cache_lock.put() function call
            cache_lock
                .put(requested_endpoint, target_response_buffer)
                .await;
        }

        info!(
            "Proxy and cache operation completed successfully for {}",
            target_addr
        );
    } else {
        warn!(
            "Proxy operation failed for {}: {:?}",
            target_addr,
            proxy_result.err().unwrap()
        );
    }
    // Decrement connection count
    {
        let mut counts = connection_counts.lock().await;
        if let Some(count) = counts.get_mut(target_addr) {
            *count = count.saturating_sub(1);
        }
    }
    Ok(())
}

async fn read_and_write(
    reader: &mut ReadHalf<'_>,
    writer: &mut WriteHalf<'_>,
    buffer: &mut Vec<u8>,
) -> io::Result<u64> {
    let mut buf = [0; 4096];
    let mut total_written = 0;

    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        buffer.extend_from_slice(&buf[0..n]);
        writer.write_all(&buf[0..n]).await?;
        total_written += n as u64;
    }

    Ok(total_written)
}
