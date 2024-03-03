# Road47

<div align="center">
    <img src="/road47logo.png">
</div>

Road47 is a versatile and high-performance proxy server designed to efficiently manage and route TCP connections across multiple backend services. Built with scalability and flexibility in mind, it offers a wide array of features tailored for modern cloud architectures, including dynamic load balancing, connection pooling, rate limiting, and resource-based routing. Road47 is ideal for applications requiring high availability, fault tolerance, and seamless integration with microservices environments.

## Features

- **Dynamic Load Balancing**: Supports various strategies such as Round Robin, Random, Least Connections, Rate Limiting, and Resource-Based to distribute traffic evenly across your services.
- **Connection Pooling**: Utilizes a connection pool to manage and reuse TCP connections, reducing latency and improving the efficiency of resource utilization.
- **Rate Limiting**: Enforces maximum request limits per target, preventing overloading of services and ensuring fair resource allocation.
- **Resource-Based Routing**: Selects targets based on their current resource usage (CPU, memory), enabling smarter routing decisions for optimized performance.
- **Caching**: Integrates an in-memory cache to store and serve frequently accessed data, significantly reducing response times and backend load.
- **Configurable and Extensible**: Easy to configure through TOML files, allowing quick setup and adjustments according to your infrastructure needs.
- **High Performance and Scalability**: Designed with performance in mind, Road47 can handle high volumes of concurrent connections and is scalable to meet the demands of growing applications.
- **Weighted Round Robin**: Distributes requests based on predefined weights assigned to each backend service, allowing more requests to be routed to higher-capacity or higher-priority services.
- **Dynamic Rate Limiting**: Adjusts request limits in real-time based on current load or other metrics, enabling more flexible and responsive load handling.

## Getting Started

### Prerequisites

- Rust and Cargo installed on your system.
- Tokio runtime for asynchronous operations.

### Installation

1. Clone the repository:

```bash
git clone https://github.com/genc-murat/road47.git
cd road47
```

2. Build the project:

```bash
cargo build --release
```

3. Configure `Config.toml` according to your requirements. Example configurations are provided in the `examples` directory.

4. Run Road47:

```bash
cargo run --release
```

## Configuration

Road47 is configured through a TOML file (`Config.toml`). Here's a basic example:

```toml
[[route]]
listen_addr = "0.0.0.0:8080"
target_addrs = ["127.0.0.1:8081", "127.0.0.1:8082"]
balance_strategy = "roundrobin"
timeout_seconds = 5
max_requests_per_target = 100
cache_ttl_seconds = 60
cache_enabled_endpoints = ["/api/v1/data"]
```

- `listen_addr`: The address on which Road47 listens for incoming connections.
- `target_addrs`: A list of backend services to which Road47 routes traffic.
- `balance_strategy`: Load balancing strategy (e.g., `roundrobin`, `random`, `leastconnections`, `ratelimiting`, `resourcebased`).
- `timeout_seconds`: Connection timeout in seconds.
- `max_requests_per_target`: Maximum number of concurrent requests per target service.
- `cache_ttl_seconds`: Time-to-live for cached entries, in seconds.
- `cache_enabled_endpoints`: List of endpoints for which caching is enabled.

## Usage

Once configured and running, Road47 will start accepting connections on the specified `listen_addr` and route them according to the defined rules and strategies.

For detailed usage and advanced configurations, refer to the documentation in the `docs` directory.

## Contributing

Contributions are welcome! Please submit pull requests with new features, improvements, or bug fixes. Ensure that your code follows the project's coding standards and includes appropriate tests.

## License

Road47 is open-sourced under the MIT License. See the LICENSE file for more details.
