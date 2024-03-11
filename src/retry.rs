use crate::config::RetryStrategyConfig;
use crate::config::StrategyType;
use crate::retry_strategy::ExponentialBackoffStrategy;
use crate::retry_strategy::FixedDelayStrategy;
use crate::retry_strategy::RetryStrategy;
use futures::future::{select_ok, BoxFuture};
use std::io::{self, Error, ErrorKind};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};

fn create_strategy(config: &RetryStrategyConfig) -> Box<dyn RetryStrategy> {
    match config.strategy_type {
        StrategyType::FixedDelay => Box::new(FixedDelayStrategy {
            delay_duration: Duration::from_millis(config.initial_delay_millis),
        }),
        StrategyType::ExponentialBackoff => Box::new(ExponentialBackoffStrategy {
            initial_delay: Duration::from_millis(config.initial_delay_millis),
            max_delay: Duration::from_secs(config.max_delay_secs),
        }),
    }
}

pub async fn connect_with_retry(
    server_addresses: &[String],
    config: RetryStrategyConfig,
) -> io::Result<TcpStream> {
    let strategy = create_strategy(&config);
    let max_attempts = config.max_attempts;
    let timeout_secs = config.timeout_secs;

    let mut attempts = 0;

    while strategy.should_retry(attempts, max_attempts) {
        let connect_futures: Vec<BoxFuture<'static, io::Result<TcpStream>>> = server_addresses
            .iter()
            .map(|address| {
                let address_cloned = address.clone();
                Box::pin(async move {
                    match timeout(
                        Duration::from_secs(timeout_secs),
                        TcpStream::connect(&address_cloned),
                    )
                    .await
                    {
                        Ok(Ok(stream)) => {
                            println!("Successfully connected to {}", address_cloned);
                            Ok(stream)
                        }
                        Ok(Err(e)) => {
                            println!("Failed to connect to {}: {}", address_cloned, e);
                            Err(e)
                        }
                        Err(_) => {
                            println!("Connection attempt to {} timed out", address_cloned);
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

        let delay = strategy.delay(attempts);
        sleep(delay).await;

        attempts += 1;
    }

    Err(Error::new(
        ErrorKind::Other,
        "Failed to connect after multiple attempts",
    ))
}
