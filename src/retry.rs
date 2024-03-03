use futures::future::{self, select_ok};
use std::io::{self, Error, ErrorKind};
use std::ops::Mul;
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout, Duration};

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
        let mut connect_futures: Vec<Pin<Box<dyn futures::Future<Output = _> + Send>>> = vec![];

        for address in server_addresses.iter() {
            let address_clone = address.clone();
            let future = Box::pin(async move {
                match timeout(
                    Duration::from_secs(timeout_secs),
                    TcpStream::connect(&address_clone),
                )
                .await
                {
                    Ok(Ok(stream)) => {
                        println!("Successfully connected to {}", address_clone);
                        Ok(stream)
                    }
                    Ok(Err(e)) => {
                        println!("Failed to connect to {}: {}", address_clone, e);
                        Err(e)
                    }
                    Err(_) => {
                        println!("Connection attempt to {} timed out", address_clone);
                        Err(Error::new(ErrorKind::TimedOut, "Connection timed out"))
                    }
                }
            });

            connect_futures.push(future);
        }

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
