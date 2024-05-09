use crate::balance::BalanceStrategy;
use mobc::Error as MobcError;
use mobc::Pool;
use road47::cache::Cache;
use road47::config::RequestModificationRule;
use road47::rate_limiter::RateLimiter;
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
use tracing::{info, warn};

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
    health_statuses: Option<Arc<Mutex<HashMap<String, bool>>>>,
    rate_limiter: Option<Arc<Box<dyn RateLimiter + Send + Sync>>>,
    rules: Option<Vec<RequestModificationRule>>,
) -> io::Result<()> {
    while let Ok((mut incoming, addr)) = listener.accept().await {
        let client_ip = addr.ip().to_string();
        if let Some(limiter) = &rate_limiter {
            if !limiter.allow(&client_ip) {
                warn!("Rate limit exceeded for IP: {}", client_ip);
                let response = "HTTP/1.1 429 Too Many Requests\r\nContent-Type: text/plain\r\nContent-Length: 33\r\n\r\nError: Rate limit exceeded.\n";
                if let Err(e) = incoming.write_all(response.as_bytes()).await {
                    warn!(
                        "Failed to send rate limit exceeded response to {}: {}",
                        client_ip, e
                    );
                }
                continue;
            }
        }
        let target_addrs_clone = Arc::clone(&target_addrs);
        let timeout_clone = timeout;
        let connection_counts_clone = Arc::clone(&connection_counts);
        let request_limits_clone = Arc::clone(&request_limits);
        let resource_endpoints_clone = resource_endpoints.as_ref().map(Arc::clone);
        let pool_clone = Arc::clone(&pool);
        let cache_clone = Arc::clone(&cache);
        let cache_enabled_endpoints_clone = cache_enabled_endpoints.clone();
        let target_weights_clone = target_weights.clone();
        let health_statuses_clone = health_statuses.as_ref().map(Arc::clone);
        let rules_for_connection = rules.clone();
        tokio::spawn(async move {
            let connection_counts_clone_for_proxy = Arc::clone(&connection_counts_clone);
            let client_ip_for_strategy = match balance_strategy {
                BalanceStrategy::IPHash => Some(client_ip.clone()),
                _ => None,
            };
            if let Some(target_addr) = balance_strategy
                .select_target(
                    target_addrs_clone,
                    connection_counts_clone,
                    request_limits_clone,
                    max_requests_per_target,
                    resource_endpoints_clone,
                    target_weights_clone,
                    health_statuses_clone,
                    client_ip_for_strategy,
                )
                .await
            {
                if let Err(e) = proxy_connection(
                    incoming,
                    &target_addr,
                    timeout_clone,
                    connection_counts_clone_for_proxy,
                    pool_clone,
                    cache_clone,
                    cache_enabled_endpoints_clone,
                    rules_for_connection,
                )
                .await
                {
                    warn!("Error proxying connection to {}: {:?}", target_addr, e);
                }
            } else {
                warn!("No target addresses available or all targets are down.");
            }
        });
    }
    Ok(())
}

async fn handle_request_modification(
    incoming: &mut TcpStream,
    rules: &[RequestModificationRule],
) -> io::Result<()> {
    let mut buffer = Vec::new();
    incoming.read_to_end(&mut buffer).await?;
    let request_str = String::from_utf8_lossy(&buffer);
    let headers_end = request_str
        .find("\r\n\r\n")
        .unwrap_or_else(|| request_str.len());
    let (header_str, body) = request_str.split_at(headers_end);
    let mut lines = header_str.lines();
    let request_line = lines.next().unwrap_or_default();
    let request_parts: Vec<&str> = request_line.split_whitespace().collect();
    if request_parts.len() < 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Malformed request line",
        ));
    }
    let (method, mut path) = (request_parts[0], request_parts[1].to_string());
    let mut headers = HashMap::new();
    for line in lines {
        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }
    for rule in rules {
        if let Some(ref contains) = rule.path_contains {
            if path.contains(contains) {
                if let Some(ref new_path) = rule.rewrite_url {
                    path = new_path.clone();
                    break;
                }
            }
        }
        for header in &rule.remove_headers {
            headers.remove(header);
        }
        headers.extend(rule.add_headers.clone());
    }
    let mut modified_request = format!("{} {} HTTP/1.1\r\n", method, path);
    for (key, value) in &headers {
        modified_request.push_str(&format!("{}: {}\r\n", key, value));
    }
    modified_request.push_str("\r\n");
    modified_request.push_str(body);
    incoming.write_all(modified_request.as_bytes()).await?;
    incoming.flush().await?;
    Ok(())
}

async fn proxy_connection(
    mut incoming: TcpStream,
    target_addr: &str,
    timeout: Duration,
    connection_counts: Arc<Mutex<HashMap<String, usize>>>,
    pool: Arc<Pool<TcpConnectionManager>>,
    cache: Arc<Mutex<Cache>>,
    cache_enabled_endpoints: Option<Vec<String>>,
    rules: Option<Vec<RequestModificationRule>>,
) -> io::Result<()> {
    if let Some(ref rules) = rules {
        if !rules.is_empty() {
            handle_request_modification(&mut incoming, rules).await?;
        }
    }
    let requested_endpoint = extract_endpoint_from_stream(&mut incoming).await?;
    if let Some(ref cache_enabled_endpoints) = cache_enabled_endpoints {
        if cache_enabled_endpoints.contains(&requested_endpoint) {
            let cache_lock = cache.lock().await;
            if let Some(cached_data) = cache_lock.get(&requested_endpoint).await {
                return send_cached_response(incoming, &cached_data).await;
            }
        }
    }
    let target = connect_to_target(&pool, timeout, target_addr).await?;
    let result = proxy_traffic_and_cache_response(
        incoming,
        target,
        target_addr,
        &connection_counts,
        cache.clone(),
        requested_endpoint.clone(),
        &cache_enabled_endpoints,
    )
    .await;
    result
}

async fn extract_endpoint_from_stream(stream: &mut TcpStream) -> io::Result<String> {
    let mut reader = BufReader::new(stream);
    let mut first_line = String::new();
    let bytes_read = reader.read_line(&mut first_line).await?;
    if bytes_read == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "EOF reached before completing read",
        ));
    }
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid HTTP request line",
        ));
    }
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