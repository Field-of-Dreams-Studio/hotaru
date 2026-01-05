# Hotaru Web Framework

![Latest Version](https://img.shields.io/badge/version-0.7.6-brightgreen)
[![Crates.io](https://img.shields.io/crates/v/hotaru)](https://crates.io/crates/hotaru)
[![GPL-3.0 License](https://img.shields.io/badge/license-GNU-blue.svg)](LICENSE)

> Small, sweet, easy framework for full-stack Rust web applications

## Former Codebase 

We rebased our code since July this year. Please refer to the following link if you are interested in our history of building the framework 

https://github.com/Redstone-D/starberry 

## 📋 Overview

Hotaru is a lightweight, intuitive web framework focused on simplicity and productivity. It supports regex-based routing, tree-structured URLs, and integrates seamlessly with the Akari templating system.

**[Example Project](https://github.com/Field-of-Dreams-Studio/hotaru)**

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
- **Client Information**: Access client IP addresses and connection details directly from handlers

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

### Using the CLI Tool (Recommended)

Install the Hotaru CLI tool:

```bash
cargo install hotaru
```

Create a new project:

```bash
hotaru new my_app
cd my_app
cargo run
```

### Manual Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hotaru = "0.7.3"
tokio = { version = "1", features = ["full"] }
```

### Optional Features

Hotaru supports the following optional features:

- **`debug`**: Enable debug logging for development and troubleshooting
- **`external-ctor`**: Use the external [`ctor`](https://crates.io/crates/ctor) crate instead of Hotaru's built-in constructor implementation

  **Note**: When enabling `external-ctor`, you must also add `ctor` to your dependencies:
  ```toml
  [dependencies]
  hotaru = { version = "0.7.3", features = ["external-ctor"] }
  ctor = "0.4.0"  # Required when external-ctor feature is enabled
  tokio = { version = "1", features = ["full"] }
  ```

  By default, Hotaru uses a built-in constructor implementation that doesn't require any external dependencies.

## 🛠️ Binary Commands

Hotaru provides a CLI tool to help you scaffold and manage projects quickly.

**⚠️ IMPORTANT**: It is **crucial** to use the CLI tool (`hotaru new` or `hotaru init`) when creating a new Hotaru project. The CLI automatically generates essential files including:
- `build.rs` - Required for asset management and resource copying
- `resource.rs` - Helper module for locating templates and static files at runtime

Without these files, your project will not be able to properly locate and serve templates or static assets. Manual setup is significantly more complex and error-prone.

### Installation

```bash
cargo install hotaru
```

**Note**: After installation, make sure Cargo's bin directory is in your PATH. If the `hotaru` command is not found, add Cargo's bin directory to your PATH:

```bash
# Linux/macOS
export PATH="$HOME/.cargo/bin:$PATH"

# Windows (PowerShell)
$env:Path += ";$env:USERPROFILE\.cargo\bin"
```

To make this permanent, add the export line to your shell configuration file (`~/.bashrc`, `~/.zshrc`, etc.).

### Available Commands

#### `hotaru new <project_name>`

Create a new Hotaru project with a complete project structure:

```bash
hotaru new my_app
```

This generates:
- `src/main.rs` - Main application entry point with a hello world endpoint
- `src/resource.rs` - Resource file locator helper module
- `build.rs` - Build script for asset management
- `Cargo.toml` - Pre-configured with Hotaru dependencies
- `templates/` - Directory for Akari HTML templates
- `programfiles/` - Directory for static assets (CSS, JS, images)

The generated project uses the latest Hotaru features including:
- `LApp!` macro for clean app initialization
- `endpoint!` macro for routing
- Proper resource management with build-time asset copying

#### `hotaru init`

Initialize an existing Cargo project with Hotaru scaffolding:

```bash
cd my_existing_project
hotaru init
```

This command adds the same structure as `hotaru new` but to your current project directory.

### Running Your Project

After creating a project:

```bash
cd my_app
cargo run
```

Your server will start at `http://127.0.0.1:3000` by default, serving a "Hello, world!" response at the root path.

### Project Structure

```
my_app/
├── Cargo.toml              # Dependencies and project metadata
├── build.rs                # Asset copying build script
├── src/
│   ├── main.rs            # Application entry point
│   └── resource.rs        # Resource locator helpers
├── templates/             # Akari HTML templates
└── programfiles/          # Static assets (CSS, JS, images)
```

The build script automatically copies `templates/` and `programfiles/` to the target directory during compilation, making them accessible to your application at runtime.

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

### Macro Notes

- `endpoint!` and `middleware!` auto-register at startup (constructor-based), so there is no manual `router.register()` step.
- Always use brace syntax `{}` and place doc comments inside the macro block.
- Optional fn-style: `pub fn name(req: HTTP) { ... }`; angle-bracket form defaults to `req`.
- Our philosophy is to wrap anything into macros to keep endpoints and middleware self-contained; see `macro_ra.md` for the minimal syntax and rationale.
- Analyzer support is planned via custom analyzer tools.

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

### Client Information

Access client IP addresses and connection details directly from your handlers:

```rust
endpoint! {
    APP.url("/api/whoami"),
    pub whoami<HTTP> {
        // Get client's full socket address (IP + port)
        match req.client_ip() {
            Some(addr) => text_response(format!("Your address: {}", addr)),
            None => text_response("Unknown client"),
        }
    }
}
```

**Available Methods:**

| Method | Return Type | Description |
|--------|-------------|-------------|
| `client_ip()` | `Option<SocketAddr>` | Client's socket address (IP + port) |
| `client_ip_or_default()` | `SocketAddr` | Returns `0.0.0.0:0` if unknown |
| `client_ip_only()` | `Option<IpAddr>` | Just the IP address, no port |
| `client_ip_only_or_default()` | `IpAddr` | Returns `0.0.0.0` if unknown |
| `server_addr()` | `Option<SocketAddr>` | Server's bound address |
| `remote_addr()` | `Option<SocketAddr>` | Alias for `client_ip()` |
| `local_addr()` | `Option<SocketAddr>` | Alias for `server_addr()` |

**Note**: When behind a reverse proxy, `client_ip()` returns the proxy's address. Use headers like `X-Forwarded-For` or `X-Real-IP` to get the original client IP.

## 📚 Examples

Check out the [example repository](https://github.com/Field-of-Dreams-Studio/hotaru-example) for:
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
- `.worker()` method now properly configures dedicated worker threads per App instance
- New `LApp!`, `LUrl!`, `LPattern!` macros for simplified lazy static declarations
- Fixed `hotaru new` and `hotaru init` to generate correct `endpoint!` macro syntax
- Built-in constructor implementation (no external `ctor` dependency required) 
- **Client IP access**: `ctx.client_ip()` and related methods for accessing socket addresses in handlers

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

## 🔮 Schedule 

| Version No. | Content | Release Date (Approx.) | 
| --- | --- | --- | 
| 0.8.0 | HTTP Outbound | Jan.2026 | 
| 0.8.3 | Using new template engine | Jan.2026 | 
| 0.8.9 | Bug Fixes | May.2026 | 

## 📚 Learn More

- **Akari Template Engine**: https://crates.io/crates/akari
- **Homepage**: https://hotaru.rs 
- **Documentation Home Page**: https://fds.rs
- **GitHub**: https://github.com/Field-of-Dreams-Studio/hotaru
- **Documentation**: https://docs.rs/hotaru 

| Video Resources | URL | 
| --- | --- | 
| Quick Tutorial | Youtube: https://www.youtube.com/watch?v=8pV-o04GuKk&t=6s <br> Bilibili: https://www.bilibili.com/video/BV1BamFB7E8n/ | 

## 🤖 AI Declaration of each Mod

We believe in transparency about AI-assisted development. Below is an honest breakdown of AI involvement per module: 

| Name | Usage of AI | Comments |
| --- | --- | --- |
| hotaru_core/app | Minimal | |
| hotaru_core/connection | Some | |
| hotaru_core/url | Minor | |
| hotaru_core/http | Minor | |
| hotaru_lib | Some | Basic API Access |
| hotaru_meta/endpoint | None | |
| hotaru_meta/middleware | None | |
| ahttpm | Major | Import Akari_macro and Improvements |
| h2per | Major | Integration of Hyper - Not stable yet |
| htmstd/cors | Minimal | |
| htmstd/session | Minimal | |

**Explanation of terms:**

| Term | Meaning |
| --- | --- |
| None | Full human code, no AI tools used |
| Minimal | AI used for autocompletion, minor suggestions, or documentation only |
| Minor | Some tabs. A few AI generated functions for logic. Testing code maybe written by AI |
| Some | Planning maybe done by AI. Overall structure written by human. Less than a third of real implementation written by AI |
| Major | Planning is done by AI. Overall structure generated by AI with supervision of human. More than a third of real implementation written by AI | 

## 📄 License

GPL-3.0 License

Copyright (c) 2024-2025 Redstone @ Field of Dreams Studio
