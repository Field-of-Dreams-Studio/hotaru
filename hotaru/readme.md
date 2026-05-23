# Hotaru Web Framework

![Latest Version](https://img.shields.io/badge/version-0.8.0-brightgreen)
[![Crates.io](https://img.shields.io/crates/v/hotaru)](https://crates.io/crates/hotaru)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> Small, sweet, easy framework for full-stack Rust web applications

License note: Already switched to MIT 

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

pub static APP: SServer = Lazy::new(|| {
    Server::new()
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
hotaru = "0.8.0"
tokio = { version = "1", features = ["full"] }
```

### Optional Features

Hotaru supports the following optional features:

- **`debug`**: Enable debug logging for development and troubleshooting
- **`https`**: HTTPS support — pulls in `hotaru_tls` and surfaces `HTTPS` / `TlsTransport` / `TlsOutboundTarget` / `TlsClientConfig`
- **`external-ctor`**: Use the external [`ctor`](https://crates.io/crates/ctor) crate instead of Hotaru's built-in constructor implementation

  **Note**: When enabling `external-ctor`, you must also add `ctor` to your dependencies:
  ```toml
  [dependencies]
  hotaru = { version = "0.8.0", features = ["external-ctor"] }
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
- `LServer!` macro for clean server initialization
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
- Optional fn-style: `pub fn name(req: HTTP) { ... }` (new in 0.7.7); angle-bracket form (hotaru blocks) defaults to `req`.
- Our philosophy is to wrap anything into macros to keep endpoints and middleware self-contained; see `macro_ra.md` for the minimal syntax and rationale.
- Analyzer support is planned via custom analyzer tools.

### Middleware

Create reusable middleware:

```rust
use htmstd::session::CookieSession;

pub static APP: SServer = Lazy::new(|| {
    Server::new()
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
- **[hotaru_trans](https://crates.io/crates/hotaru_trans)** - Procedural macros for endpoint! and middleware! 
- **[hotaru_http](https://crates.io/crates/hotaru_http)** - HTTP implementation for Hotaru 
- **[hotaru_tls](https://crates.io/crates/hotaru_tls)** - TLS/HTTPS implementation for Hotaru 
- **[hotaru_lib](https://crates.io/crates/hotaru_lib)** - Utility functions (compression, encoding, etc.)
- **[htmstd](https://crates.io/crates/htmstd)** - Standard middleware library (CORS, sessions)

## 📋 Changelog 

### 0.8.0 (Current)
- **Client / outpoint runtime**: new `Client<TS>` for outbound traffic, mirroring `Server<TS>`. Includes `Client::request_fn`, `Client::call_fn`, `Client::call_url` for one-shot and persistent outpoint invocation.
- **`outpoint!` macro**: client-side counterpart to `endpoint!`. The user body becomes the outermost middleware; the `send;` marker triggers the registered chain (terminating in `<P as Protocol>::send(ctx).await`).
- **`run!` and `call!` macros**: invocation-style sugar — `run!(APP<HTTP>::name, request)` -> `APP.request_fn::<HTTP>("name", request)`; `call!(APP<HTTP>::name)` -> `APP.call_fn::<HTTP>("name")` (plus `: "/path"` form for `call!`).
- **Protocol trait reshape**: channel-based `open_channel` / `handle(channel, runtime, root)` / `send(ctx) -> ctx`, plus a per-protocol `install_channel(&mut ctx, channel)` bridge. `type Context: RequestContext<Channel = Self::Channel>` pins channel-type alignment.
- **RequestContext trait**: `Default` supertrait; declares `type Channel: Channel` as a type anchor only (channel lives as a private field on each concrete context — no trait-exposed accessor). Adds `inject_request` / `into_response`. `type Error: ProtocolError` is the single source of truth for chain errors.
- **Result-typed execution chain**: middleware, final handlers, `ExecutionChain::run`, and `UrlNode::run` all return `Result<C, <C as RequestContext>::Error>` — no boxing at the chain boundary.
- **Named access points**: every registered endpoint/outpoint has an explicit name. `ProtocolEntry::register(name, path, step_names, binding, config)` is the single canonical funnel; `AccessPointTable` per protocol entry refreshes Node-variant entries on rebind. `Server::url(url, name, ...)` / `Client::url(url, name, ...)` are URL-first.
- **Instance-based transports**: `TransportSpec::Inbound` / `Outbound` replace the old `Accepter` / `Connector` shape. `TcpInbound::bind(target)` and `TcpOutbound::build(target)` materialize once per `Server`/`Client`.
- **HTTPS feature**: new `https` cargo feature on the umbrella `hotaru` crate forwards to `hotaru_http/tls`, surfacing `HTTPS`, `TlsTransport`, `TlsOutboundTarget`, `TlsClientConfig`. `HTTPS = Http1Protocol<TlsStream, TlsTransport>`.
- **`Channel` trait + `ProtocolFlow`**: per-exchange close semantics (HTTP/1 closes the connection; multiplexed protocols may close only the stream). `Channel: Clone` is the framework's sharing contract; protocols are free to back the handle with `Arc<...>` internally.
- New `LServer!`, `LClient!`, `LUrl!`, `LPattern!` macros for simplified lazy static declarations (replaces the old `LApp!`). Defaults to `TcpTransport`; HTTPS clients with `Client<TlsTransport>` currently need a direct `Lazy::new(...)` declaration until the `LClient!` type-parameter passthrough lands.

### 0.7.x
- Multi-protocol support (HTTP, WebSocket, custom TCP)
- Enhanced security controls with HttpSafety
- Improved middleware system with protocol inheritance
- Performance optimizations in URL routing
- Comprehensive security testing
- `.worker()` method now properly configures dedicated worker threads per Server instance
- Fixed `hotaru new` and `hotaru init` to generate correct `endpoint!` macro syntax
- Built-in constructor implementation (no external `ctor` dependency required)
- Fn-style blocks: New syntax `pub fn name(req: HTTP) { ... }` for `endpoint!` and `middleware!` macros (original hotaru blocks syntax preserved)
- Bug fix for URL routing

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

We believe in transparency about AI-assisted development. The framework is governed jointly by two maintainer groups using a shared four-tier system that prioritizes understanding over line counts.

### Maintained by: PMINE/Research

| Name | Tier | Comments |
| --- | --- | --- |
| hotaru_core/app | Author-Owned | |
| hotaru_core/connection | Author-Owned | |
| hotaru_core/executable | Author-Owned | |
| hotaru_core/url | Author-Owned | |
| hotaru_core/protocol | Author-Owned | |
| hotaru_http/trails | Co-Authored | |
| hotaru_http/* | Human-Led | |
| hotaru_mqtt/broker | Co-Authored | |
| hotaru_mqtt/traits | Co-Authored | |
| hotaru_mqtt/* | Human-Led | |
| hotaru_lib | Human-Led | Basic API Access |
| h2per | Co-Authored | Integration of Hyper - Not stable yet |
| htmstd/cors | Human-Led | |
| htmstd/session | Human-Led | |

### Maintained by: Project-StarFall

| Name | Tier | Comments |
| --- | --- | --- |
| hotaru_trans/endpoint | Author-Owned | Proof and language design must be fully understood by humans |
| hotaru_trans/outpoint | Author-Owned | Proof and language design must be fully understood by humans |
| hotaru_trans/middleware | Author-Owned | Proof and language design must be fully understood by humans |
| hotaru_trans/cors | Co-Authored | Trivial user-level abstraction |
| ahttpm | Co-Authored | Imports akari_macro plus improvements |
| SFX | Co-Authored | Trivial user-level abstraction |
| akari | External | https://crates.io/crates/akari |
| akari_lang | External (TBD) | |
| akari_macro | External | https://crates.io/crates/akari |

### Shared term meanings

| Term | Meaning |
| --- | --- |
| **Forbidden** | The intelligence work in this module — design decisions, proof obligations, language semantics, novel logic — is authored by humans. AI is not used for this content (the mechanical/test/doc carve-out in operating rule 2 still applies). Reserved for modules where the work *is* the thinking, not the typing. |
| **Author-Owned** | AI may assist with drafts and completion, but the committed code reads as the author's own throughout. A reviewer should not be able to tell where AI helped. The module signals "a human owns the design and the prose." |
| **Human-Led** | The human authored the structure and the load-bearing pieces; AI filled in helpers, repetitive sections, or boilerplate. Some sections may visibly bear AI's hand, but the design choices and non-trivial logic are clearly human. The author can defend every part without re-consulting AI. |
| **Co-Authored** | AI participated substantively in both design exploration and implementation. The human author has internalized the result and can defend, modify, and debug without re-prompting. Appropriate for well-understood patterns and third-party integrations. |

The understanding requirement is uniform across Author-Owned, Human-Led, and Co-Authored: the author can explain any line, modify surrounding code without AI help, and walk a reviewer through the code on request. The tiers differ only in where AI's voice is allowed to show through, not in what the author owes the team.

### Operating rules

1. **No quantification.** Tiers describe the kind of collaboration, not the amount of AI-authored code. Counting lines is brittle and incentivizes the wrong behavior.
2. **Tests, documentation, and mechanical typing are permitted in every tier — including Forbidden.** AI assistance is allowed across all modules for: unit tests; doc comments and inline prose; and mechanical typing where the design decision has already been made by a human and AI is only writing it out (e.g., applying a settled pattern across similar cases, expanding hand-authored pseudocode, regenerating a table from a spec). The defining criterion is that no *intelligence work* is delegated to AI — design, proof, and semantics decisions remain with the human. The author remains responsible for understanding what was generated.
3. **Reviewer-driven understanding check.** Any reviewer may flag a PR with "this doesn't feel author-owned" — regardless of the module's tier. The author clears the flag by demonstrating understanding in PR comments or a short walkthrough. Flags are requests for evidence, not accusations.
4. **Smell-test threshold scales with tier.** Author-Owned code is flagged if any section visibly reads as AI-generated. Human-Led is flagged if the structural code reads as AI-generated or AI's hand pervades rather than appearing locally. Co-Authored is flagged only if the author cannot defend the code in review.
5. **Tier reflects the work, not preferences.** Maintainers set tiers based on the nature of the module. If the character of a module changes, the tier is re-set rather than stretched.
6. **External code is outside this policy.** AI-authored code arriving through a third-party crate is governed by that crate's own conventions, transparently linked.

## 📄 License

MIT License

Copyright (c) 2024-2026 @ Field of Dreams Studio 

[Project-StarFall](https://sf.fds.moe), [PMINE](https://pmine.fds.moe) 
