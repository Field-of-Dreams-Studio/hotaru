# hotaru_core

[![Crates.io](https://img.shields.io/crates/v/hotaru_core)](https://crates.io/crates/hotaru_core)

Core framework for building multi-protocol web applications in Rust. Provides HTTP/HTTPS server implementation, URL routing, middleware system, and protocol abstraction layer.

## Features

- **HTTP/1.1 Server** - Full HTTP protocol implementation with chunked encoding, compression, cookies
- **Multi-Protocol Support** - Abstract protocol layer for HTTP, WebSocket, custom TCP protocols
- **Flexible Routing** - Priority-based URL matching with regex patterns and dynamic parameters
- **Middleware System** - Composable request/response processing pipeline
- **TLS Support** - Built-in HTTPS with rustls
- **Security** - Configurable size limits, method filtering, and input validation
- **Async/Await** - Built on Tokio for high-performance async I/O

## Architecture

```
Application
  ├─ Protocol Layer (HTTP, WebSocket, TCP)
  ├─ URL Routing Tree
  ├─ Middleware Chain
  └─ Request/Response Context
```

## Usage

This is the core framework library. For application development, use the [hotaru](https://crates.io/crates/hotaru) crate which provides a more convenient API.

```toml
[dependencies]
hotaru_core = "0.7.5"
```

## Documentation

For detailed documentation and examples, see the [Hotaru web framework](https://crates.io/crates/hotaru).

## License

GPL-3 License

## Part of Hotaru Framework

This is the core crate for the [Hotaru web framework](https://crates.io/crates/hotaru).

Learn more: https://fds.rs
