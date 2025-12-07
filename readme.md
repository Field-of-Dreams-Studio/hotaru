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

### Bug Fixes & Syntax Sugar added in 0.7.6

##### [1] Now the worker() function for APP is now useful 

The `.worker()` method now properly configures the number of worker threads for the application. Each App instance creates its own independent tokio runtime with the specified worker count when `run()` is called. This setting is independent of any outer runtime configuration.

```rust
#[tokio::main]
async fn main() {
    let app = App::new()
        .worker(4)  // App runs with 4 dedicated worker threads
        .binding("127.0.0.1:3000")
        .build();
    app.run().await;
}
``` 

##### [2] LApp!, LUrl!, and LPattern! macros are available for simplified lazy static declarations 

New convenience macros simplify creating lazy static instances with less boilerplate:

```rust
use hotaru::prelude::*;

// Old way
pub static APP: SApp = Lazy::new(|| {
    App::new().build()
});

// New way with LApp! macro
LApp!(APP = App::new().build());

// Also works for URLs and patterns
LUrl!(HOME_URL = Url::new("/home"));
LPattern!(API_PATTERN = PathPattern::new("/api/*"));
```

The `L` prefix stands for "Lazy/Load" - these macros automatically wrap your expression in `Lazy::new(|| ...)` and create a public static with the appropriate type (`SApp`, `SUrl`, `SPattern`).

**Benefits:**
- Less boilerplate - no manual `Lazy::new(|| ...)` wrapper
- Clear intent - `LApp!` immediately signals "lazy app instance"
- Consistent pattern across all lazy static declarations
- Educational - shows the assignment pattern while hiding ceremony

##### [3] Bug Fixes in `hotaru new` and `hotaru init` commands

Fixed template generation issues in the hotaru CLI tool:

- **Fixed old macro usage**: The generated template now uses the correct `endpoint!` macro syntax instead of the old syntax `#[url]` attribute
- **Updated to use LApp!**: Generated projects now use `LApp!(APP = App::new().build())` for cleaner initialization
- **Improved template structure**: The hello world template now follows current best practices with proper imports and macro usage

Generated projects now compile successfully out of the box without manual fixes.

##### [4] Built-in constructor implementation in hotaru_meta

Hotaru now includes its own implementation of the constructor pattern (similar to the `ctor` crate) to ensure projects compile without additional dependencies.

**By default**, Hotaru uses a built-in constructor implementation that supports Linux, macOS, and Windows. This is production-ready and convenient for most use cases.

**If you encounter any issues** or want the battle-tested `ctor` crate instead, you can switch to the external implementation:

```toml
[dependencies]
hotaru = { version = "0.7.6", features = ["external-ctor"] }
ctor = "0.4.0"  # Required when using external-ctor feature
```

The built-in implementation is provided for convenience and is production-ready for the supported platforms. However, if you experience any platform-specific issues, switching to the external `ctor` crate is recommended. 

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
