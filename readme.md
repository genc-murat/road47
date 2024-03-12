# Road47

<div align="center">
    <img src="/road47logo.png">
</div>

# Road47: A High-Performance, Rust-Based Load Balancer and Proxy

Road47 is an innovative, feature-rich load balancer and proxy solution designed to optimize the distribution of traffic across multiple servers, ensuring high availability and reliability of services. Written in Rust, Road47 leverages the language's safety, performance, and concurrency features, making it an ideal choice for handling high-throughput and low-latency network applications.

## Features

- **Load Balancing Strategies**: Supports multiple algorithms including Round Robin, Random, Least Connections, Rate Limiting, Resource-Based, Weighted Round Robin, Dynamic Rate Limiting, and IP Hash, allowing administrators to choose the most suitable strategy based on their specific use case.
- **Dynamic Configuration**: Configuration can be updated on the fly without restarting the service, minimizing downtime and enabling seamless adjustments to changing load patterns.
- **Resource Usage Monitoring**: Integrates with endpoints to monitor CPU and memory usage, enabling Resource-Based balancing decisions that consider the current load on target servers.
- **Connection Management**: Maintains connection counts and enforces request limits per target, with support for dynamic rate limiting based on current load, ensuring fair resource allocation and preventing server overload.
- **Health Checking**: Periodically checks the health of target servers to ensure traffic is only routed to healthy instances, enhancing the overall reliability of the service.
- **Caching**: Implements an LRU (Least Recently Used) cache to store and serve frequently accessed data, reducing latency and offloading traffic from backend servers.
- **Retry Strategies**: Offers configurable retry logic that includes exponential backoff and timeout settings, improving the resilience of the system in the face of temporary network failures or server unavailability.
- **TCP Connection Management**: Utilizes a custom TCP connection manager to efficiently manage connections to backend servers, including support for connection pooling and retry strategies for failed connection attempts.
- **Async/Await Support**: Fully asynchronous architecture powered by Tokio, enabling non-blocking I/O operations that scale efficiently across cores.
- **Logging and Monitoring**: Comprehensive logging for debugging and monitoring, facilitating the diagnosis of issues and performance optimization.
- **Rate Limiting**: Implements various rate limiting strategies such as Fixed Window, Sliding Window, Token Bucket, and Leaky Bucket, allowing fine-grained control over the rate at which requests are processed and ensuring the stability of backend services under heavy load conditions.
- **Endpoint Extraction for Caching**: Automatically extracts requested endpoints from incoming requests, enabling more efficient caching strategies by storing and serving cached data based on specific endpoints.
- **Automatic Connection and Request Count Management**: Dynamically manages connection and request counts per target server, automatically adjusting to ensure balanced distribution of traffic and preventing any single server from becoming overloaded.
- **Enhanced Caching Mechanisms**: Beyond basic LRU caching, the system now supports conditional caching based on request endpoints, allowing specific responses to be cached and served directly, reducing load on backend services and improving response times for end-users.

## Getting Started

1. **Prerequisites**:
   - Rust and Cargo installed on your machine.
   - Configuration file (`Config.toml`) prepared according to your environment and requirements.

2. **Installation**:
   Clone the repository and build the project using Cargo:
   ```bash
   git clone https://github.com/genc-murat/road47.git
   cd road47
   cargo build --release
   ```

3. **Configuration**:
   Edit the `Config.toml` file to set up your routes, load balancing strategies, target servers, and other settings like health check endpoints, retry strategies, and cache configurations.

4. **Running Road47**:
   Start the Road47 service with:
   ```bash
   cargo run --release
   ```

5. **Monitoring and Logging**:
   Monitor the logs for any errors or important messages. Adjust the log level in the configuration file or environment variables to control the verbosity.

## Conclusion

Road47 stands out as a powerful, flexible solution for modern load balancing and proxying needs. Its use of Rust ensures that it is not only efficient and fast but also safe and reliable. Whether you're handling microservices architecture, a large distributed system, or simply need a high-performance reverse proxy, Road47 offers the features and flexibility to support your infrastructure's needs.

## Contributing

Contributions are welcome! Please submit pull requests with new features, improvements, or bug fixes. Ensure that your code follows the project's coding standards and includes appropriate tests.

## License

Road47 is open-sourced under the MIT License. See the LICENSE file for more details.
