use std::io;
use std::ops::Mul;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

pub async fn connect_with_retry(
    server_addresses: &[String],
    max_attempts: usize,
    max_delay_secs: u64,
    initial_delay_millis: u64,
) -> io::Result<TcpStream> {
    let mut attempts = 0;
    let mut delay = Duration::from_millis(initial_delay_millis);

    while attempts < max_attempts {
        for address in server_addresses.iter() {
            match TcpStream::connect(address).await {
                Ok(stream) => return Ok(stream),
                Err(e) => {
                    eprintln!("Failed to connect to {}: {:?}", address, e);
                }
            }
        }

        attempts += 1;
        if attempts >= max_attempts {
            break;
        }

        // Exponansiyel geri çekilme uygula
        delay = std::cmp::min(delay.mul(2), Duration::from_secs(max_delay_secs));
        sleep(delay).await;
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "Failed to connect after multiple attempts",
    ))
}
