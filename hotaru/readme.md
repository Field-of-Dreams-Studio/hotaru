# Hotaru Web Framework

![Latest Version](https://img.shields.io/badge/version-0.7.3-brightgreen)
[![Crates.io](https://img.shields.io/crates/v/hotaru)](https://crates.io/crates/hotaru)
[![GPL-3.0 License](https://img.shields.io/badge/license-GNU-blue.svg)](LICENSE)

> Small, sweet, easy framework for full-stack Rust web applications

## 📋 Overview

Hotaru is a lightweight, intuitive web framework focused on simplicity and productivity. It supports regex-based routing, tree-structured URLs, and integrates seamlessly with the Akari templating system.

**[Example Project](https://github.com/Field-of-Dreams-Studio/hotaru-example)**

MSRV: 1.86

## ✨ Key Features

- **Simple API**: Intuitive request/response handling with minimal boilerplate
- **Full-Stack**: Built-in template rendering with Akari templates
- **Flexible Routing**: Support for regex patterns, literal URLs, and nested routes
- **Asynchronous**: Built with Tokio for efficient async handling
- **Form Handling**: Easy processing of form data and file uploads
- **Middleware Support**: Create reusable request processing chains
- **Multi-Protocol Support**: Handle HTTP/HTTPS, WebSocket, and custom TCP protocols
- **Security**: Built-in request validation, size limits, and safety controls

## 🚀 Quick Start

```rust
use hotaru::prelude::*;

pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3000")
        .build()
});

#[tokio::main]
async fn main() {
    APP.clone().run().await;
}

endpoint! {
    APP.url("/"),
    pub index<HTTP> {
        text_response("Hello, Hotaru!")
    }
}
```

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hotaru = "0.7.3"
tokio = { version = "1", features = ["full"] }
```

## 🎯 Core Concepts

### Endpoints

Define routes with the `endpoint!` macro:

```rust
endpoint! {
    APP.url("/users/<int:id>"),
    pub get_user<HTTP> {
        let user_id = req.param("id");
        json_response(json!({ "id": user_id }))
    }
}
```

### Middleware

Create reusable middleware:

```rust
use htmstd::session::CookieSession;

pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3000")
        .append_middleware::<CookieSession>()
        .build()
});
```

### Templates

Render HTML with Akari:

```rust
endpoint! {
    APP.url("/profile"),
    pub profile<HTTP> {
        let data = json!({ "name": "Alice" });
        template_response("profile.html", data)
    }
}
```

### HTTP Safety Configuration

Configure request validation per endpoint:

```rust
endpoint! {
    APP.url("/upload"),
    config = [HttpSafety::new()
        .with_max_body_size(50 * 1024 * 1024)  // 50MB
        .with_allowed_methods(vec![HttpMethod::POST])
    ],
    pub upload<HTTP> {
        // Handle file upload
    }
}
```

## 📚 Examples

Check out the [example repository](https://github.com/Field-of-Dreams-Studio/hotaru) for:
- Basic routing and handlers
- Form processing and file uploads
- Session management with cookies
- CORS configuration
- Multi-protocol applications

## 🔧 Crate Ecosystem

Hotaru is built on a modular architecture:

- **[hotaru](https://crates.io/crates/hotaru)** - Main framework with convenient API
- **[hotaru_core](https://crates.io/crates/hotaru_core)** - Core protocol and routing engine
- **[hotaru_meta](https://crates.io/crates/hotaru_meta)** - Procedural macros for endpoint! and middleware!
- **[hotaru_lib](https://crates.io/crates/hotaru_lib)** - Utility functions (compression, encoding, etc.)
- **[htmstd](https://crates.io/crates/htmstd)** - Standard middleware library (CORS, sessions)

## 📋 Changelog

### 0.7.x (Current)
- Multi-protocol support (HTTP, WebSocket, custom TCP)
- Enhanced security controls with HttpSafety
- Improved middleware system with protocol inheritance
- Performance optimizations in URL routing
- Comprehensive security testing

### 0.6.x
- Protocol abstraction layer
- Request context improvements
- Standard middleware library (htmstd)
- Cookie-based session management

### 0.4.x and earlier
- Async/await support with Tokio
- Akari templating integration
- Cookie manipulation APIs
- File upload handling
- Form data processing improvements

## 🔮 Roadmap

- WebSocket support improvements
- HTTP/2 protocol implementation
- GraphQL integration
- Advanced caching strategies
- Performance benchmarking suite

## 📚 Learn More

- **Akari Template Engine**: https://crates.io/crates/akari
- **Homepage**: https://fds.rs
- **GitHub**: https://github.com/Field-of-Dreams-Studio/hotaru
- **Documentation**: https://docs.rs/hotaru

## 📬 Get Involved

- **GitHub Issues**: https://github.com/Field-of-Dreams-Studio/hotaru/issues 
- **Discussions**: https://github.com/Field-of-Dreams-Studio/hotaru/discussions 
- **Email**: redstone@fds.moe
- **Discord Group**: https://discord.gg/Y6b9KRUCux 
- **QQ Group**: 860691370  
- **Join FDS**: https://forms.office.com/Pages/ResponsePage.aspx?id=DQSIkWdsW0yxEjajBLZtrQAAAAAAAAAAAAMAAC6BwJ5UQ0lQUzdMTjhGR1g3SElLTFdHQUlJV0hFMS4u 

## 📄 License

GPL-3.0 License

Copyright (c) 2025 @ Field of Dreams Studio
