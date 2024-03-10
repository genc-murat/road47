use futures::future::{select_ok, BoxFuture};
use std::io::{self, Error, ErrorKind};
use std::ops::Mul;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};

pub async fn connect_with_retry(
    server_addresses: &[String],
    max_attempts: usize,
    max_delay_secs: u64,
    initial_delay_millis: u64,
    timeout_secs: u64,
) -> io::Result<TcpStream> {
    let mut attempts = 0;
    let mut delay = Duration::from_millis(initial_delay_millis);

    while attempts < max_attempts {
        let connect_futures: Vec<BoxFuture<'static, io::Result<TcpStream>>> = server_addresses
            .iter()
            .map(|address| {
                let address = address.clone();
                Box::pin(async move {
                    match timeout(
                        Duration::from_secs(timeout_secs),
                        TcpStream::connect(&address),
                    )
                    .await
                    {
                        Ok(Ok(stream)) => {
                            println!("Successfully connected to {}", address);
                            Ok(stream)
                        }
                        Ok(Err(e)) => {
                            println!("Failed to connect to {}: {}", address, e);
                            Err(e)
                        }
                        Err(_) => {
                            println!("Connection attempt to {} timed out", address);
                            Err(Error::new(ErrorKind::TimedOut, "Connection timed out"))
                        }
                    }
                }) as BoxFuture<'static, io::Result<TcpStream>>
            })
            .collect();

        match select_ok(connect_futures).await {
            Ok((stream, _)) => return Ok(stream),
            Err(e) => println!("All connection attempts failed on this iteration: {:?}", e),
        }

        attempts += 1;
        if attempts < max_attempts {
            sleep(delay).await;
            delay = std::cmp::min(delay.mul(2), Duration::from_secs(max_delay_secs));
        }
    }

    Err(Error::new(
        ErrorKind::Other,
        "Failed to connect after multiple attempts",
    ))
}
