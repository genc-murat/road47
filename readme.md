
# Road47 Load Balancer/Proxy Server

This project implements a load balancer/proxy server capable of distributing incoming connections to a pool of target servers using various balancing strategies. It is designed to handle high availability and efficient distribution of network traffic among multiple servers, improving the scalability and reliability of web applications.

## Features

- **Multiple Balancing Strategies**: Supports Round Robin, Random, Least Connections, Rate Limiting, and Resource-Based balancing strategies to adapt to different use cases and traffic patterns.
- **Resource Usage Monitoring**: Fetches and utilizes CPU and memory usage data from target servers to make informed decisions in the Resource-Based strategy.
- **Dynamic Request Limiting**: Enforces maximum request limits per target server to prevent overloading and ensure fair distribution.
- **Asynchronous and Concurrent**: Built with Tokio for asynchronous IO, allowing for high concurrency and non-blocking network communication.
- **Configuration via TOML File**: Easily configurable server settings, including target servers, listen addresses, balancing strategies, and more.

## Getting Started

### Prerequisites

- Rust Programming Language and Cargo package manager.
- `tokio` and `reqwest` for async programming and HTTP requests, respectively.
- `serde` for serialization and deserialization of data.
- `mobc` for connection pooling.

### Installation

1. Clone the repository:
   ```sh
   git clone https://github.com/genc-murat/road47.git
   ```

2. Navigate to the project directory:
   ```sh
   cd road47
   ```

3. Build the project:
   ```sh
   cargo build --release
   ```

4. Configure your `config.toml` file according to your setup.

### Running

Execute the binary with:

```sh
cargo run --release
```

## Configuration

Edit the `config.toml` file to set up your listening addresses, target servers, balancing strategy, etc. Here is an example configuration:

```toml
[[route]]
listen_addr = "127.0.0.1:8080"
target_addrs = ["192.168.1.1:80", "192.168.1.2:80"]
balance_strategy = "roundrobin"
timeout_seconds = 5
max_requests_per_target = 100
resource_endpoints = ["http://192.168.1.1/resource", "http://192.168.1.2/resource"]
```

## Supported Balancing Strategies

- **RoundRobin**: Distributes requests evenly across all servers.
- **Random**: Selects a target server randomly for each request.
- **LeastConnections**: Prefers servers with the least active connections.
- **RateLimiting**: Limits the number of requests to each server within a time window.
- **ResourceBased**: Chooses servers based on their current CPU and memory usage.

## Contributing

Contributions are welcome! Please feel free to submit pull requests, report bugs, or suggest new features.

## License

Distributed under the MIT License. See `LICENSE` for more information.
