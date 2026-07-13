The Hotaru 0.8 era starts from 23/May/2026.

# Hotaru Web Framework

![Latest Version](https://img.shields.io/badge/version-0.8.4-brightgreen)
[![Crates.io](https://img.shields.io/crates/v/hotaru)](https://crates.io/crates/hotaru)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.txt)

## Overview

<!--The name 'Hotaru' comes from the Japanese Character '蛍（ほたる）' represents the firefly.--> 

> Small, sweet, easy framework with a protocol-neutral, no_std-ready core 

**[Official Website](https://hotaru.rs)** | **[Example Project](https://github.com/Field-of-Dreams-Studio/hotaru-example)**

MSRV: 1.88

### Stability in 0.8.x

The **tokio + HTTP** stack (default features `trans`, `http`, `tokio`) is the tested, supported path and is safe for production use today.

Everything else is **experimental** and will stabilize by 0.8.7:

- `RuntimeSpec` trait surface (`hotaru_rt_tokio` is the supported default; `hotaru_rt_embassy` is experimental)
- `no_std` builds of `hotaru_core` (Cortex-M / RISC-V bare-metal, CI-verified and connected to experimental embedded backend crates, but not yet production-validated on hardware)
- IO adapter crates: `hotaru_io_futures` ships as a standalone crate (limited real-world use). `hotaru_io_embedded` lives in the workspace and is still experimental and unpublished (crates.io). The `hotaru` facade exposes `EmbeddedIo` through its optional `io_embedded` feature.
- Embassy runtime backend (`hotaru_rt_embassy`, experimental)

If you are shipping something now, stick with the `tokio` default and revisit the experimental paths as they land.

## Key Features

<!--TODO: Make sure change this in 0.8.7-->

- **Multi-Protocol**: HTTP/1.1 and HTTPS (TLS) ship out of the box. The `Protocol` trait is an open extension point for custom TCP-based protocols (WebSocket, MQTT, and other frames), though no non-HTTP protocol ships in this workspace today
- **Server + Client**: Endpoints for inbound traffic, outpoints for outbound. Same protocol trait, same routing, same middleware
- **Runtime-Neutral Core**: `hotaru_core` speaks to any async runtime through the `RuntimeSpec` trait. Tokio ships today via `hotaru_rt_tokio`; other runtimes can plug in via the same sibling-crate pattern. IO adapters are further along, with `hotaru_io_tokio`, `hotaru_io_futures`, and the experimental in-workspace `hotaru_io_embedded`
- **`no_std`-Ready Core**: `hotaru_core` builds bare-metal on Cortex-M4/M7 and RISC-V (with atomics) under `alloc`. CI verified on `thumbv7em-none-eabihf` and `riscv32imac-unknown-none-elf`
- **Sync main**: `fn main() { run_server!(APP); }`. No `async fn main`, no `#[tokio::main]`
- **Ergonomic Macros**: `endpoint!` / `outpoint!` / `middleware!` DSL in three flavors (`trans`, `semi-trans`, `attr`)
- **Full-Stack**: Akari template rendering, form/URL-encoded body parsing, session cookies, HTTP body compression (gzip / deflate / brotli / zstd) all built in
- **Flexible Routing**: Regex, literal, and pattern segments (`<int:id>`, `<uuid:token>`, `<**path>`) with a tree walker

## Quick Start

```rust
use hotaru::prelude::*;
use hotaru::http::*;

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3003")
        .single_protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        .build()
);

fn main() {
    run_server!(APP);
}

endpoint! {
    APP.url("/"),
    pub index<HTTP> {
        text_response("Hello, Hotaru!")
    }
}
```

`run_server!(APP)` builds a tokio runtime, blocks the current thread, and shuts down on Ctrl+C. No `async fn main`, no `#[tokio::main]`. See [Core Concepts](#core-concepts) for the sibling macros (`run_server_until!`, `run_server_no_block!`, `run_server_no_block_until!`) when you need a custom stop source or multi-server orchestration.

## Installation

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
hotaru = "0.8.3"
tokio = { version = "1", features = ["full"] }
```

### Optional Features

Default features: `trans`, `http`, `tokio`. Cargo's additive feature unification means sub-features pull in their prerequisites automatically — you never have to enable a base feature by hand.

**Protocol stack**

- **`http`** *(default-on)*: HTTP/1.1 stack (`hotaru_http` + `ahttpm`). Opt out with `default-features = false` for protocol-only builds (e.g. gRPC-only deployments) — `hotaru::http::*`, `HTTP`, `HttpContext`, `HttpRequest`, `HttpResponse`, etc. then disappear from the crate surface.
- **`tokio`** *(default-on)*: Tokio runtime + TCP/IO defaults for the umbrella crate (`Server`, `Client`, `Url`, `S*` aliases, `TcpTransport`, `TokioRuntime`). If you disable default features but still use those defaults, re-enable `tokio`.
- **`https`**: TLS/HTTPS support — surfaces `HTTPS`, `TlsTransport`, `TlsOutboundTarget`, `TlsClientConfig`. Implies `http`.
- **`http_compression`**: HTTP body codecs for `Content-Encoding` (gzip / deflate / brotli / zstd). Off by default because `brotli` + `zstd` together add ~7 s to a clean build. Implies `http`. Without this feature, `ContentCoding::decode_compressed` / `encode_compressed` return `io::ErrorKind::Unsupported` for compressed bodies.

**Endpoint macro flavor** — pick one (see Core Concepts):

- **`trans`** *(default)* — bang macro with hotaru-blocks body
- **`semi-trans`** — stacked attributes above an `fn`
- **`attr`** — single attribute with args

**Misc**

- **`debug`**: Enable debug logging for development and troubleshooting.
- **`external-ctor`**: Use the external [`ctor`](https://crates.io/crates/ctor) crate instead of Hotaru's built-in constructor implementation. When enabling, you must also add `ctor` to your dependencies:
  ```toml
  [dependencies]
  hotaru = { version = "0.8.3", features = ["external-ctor"] }
  ctor = "0.4.0"
  tokio = { version = "1", features = ["full"] }
  ```

**Example — HTTPS server with body compression:**

```toml
[dependencies]
hotaru = { version = "0.8.3", features = ["https", "http_compression"] }
tokio = { version = "1", features = ["full"] }
```

**Example — gRPC-only (no HTTP):**

```toml
[dependencies]
hotaru = { version = "0.8.3", default-features = false, features = ["trans", "tokio"] }
hotaru_grpc = "..."
tokio = { version = "1", features = ["full"] }
```

## Binary Commands

Use the CLI to scaffold projects — it generates `build.rs` for asset copying and `src/resource.rs` for runtime template/static lookup, which are non-trivial to wire up by hand.

```bash
cargo install hotaru                   # install the CLI (see Installation above)
hotaru new my_app                      # scaffold a new project
hotaru init                            # or scaffold into the current Cargo crate
cd my_app && cargo run                 # serves http://127.0.0.1:3003
```

### Project Structure

```
my_app/
├── Cargo.toml              # Dependencies and project metadata
├── build.rs                # Asset copying build script
├── src/
│   ├── main.rs            # Application entry point with LServer! + endpoint!
│   └── resource.rs        # Resource locator helpers
├── templates/             # Akari HTML templates
└── programfiles/          # Static assets (CSS, JS, images)
```

The build script copies `templates/` and `programfiles/` to the target directory at compile time so they're accessible at runtime.

## Core Concepts

### Endpoints

Three macro flavors, enabled by the `trans` / `semi-trans` / `attr` cargo features. Pick one per project; **`trans` is the default**. All three register the same route at startup; they only differ in syntax.

**`trans` (default) — bang macro with hotaru-blocks body:**

```rust
endpoint! {
    APP.url("/users/<int:id>"),
    pub get_user<HTTP> {
        let user_id = req.param("id").unwrap_or_default();
        akari_json!({ id: user_id })
    }
}
```

**`semi-trans` — stacked attributes above an `fn`:**

```rust
#[endpoint]
#[url("/users/<int:id>")]
pub fn get_user<HTTP>() {
    let user_id = req.param("id").unwrap_or_default();
    akari_json!({ id: user_id })
}
```

**`attr` — single attribute with args:**

```rust
#[endpoint("/users/<int:id>")]
pub fn get_user<HTTP>() {
    let user_id = req.param("id").unwrap_or_default();
    akari_json!({ id: user_id })
}
```

> `akari_json!` is the JSON-response macro re-exported via `hotaru::prelude`; it already wraps `json_response(...)` so callers don't compose the two. Keys are bare idents (not `"..."`). `req.param(...)` returns `Option<String>`.

### Macro Notes

- Endpoints and middleware auto-register at startup — no manual `router.register()`.
- `trans` form: brace syntax `{}` with doc comments inside the block; angle-bracket body defaults to `req`. Optional fn-style `pub fn name(req: HTTP) { ... }` is also accepted.
- Remaining readme examples use `trans`. To switch, set `default-features = false` on the `hotaru` dependency and turn on the flavor you want, e.g. `hotaru = { version = "0.8.3", default-features = false, features = ["semi-trans", "http", "tokio"] }`. Cargo feature unification would otherwise keep `trans` on alongside it; remember to re-add `http` and `tokio` since `default-features = false` also drops the default HTTP stack and Tokio facade defaults.
- See `macro_ra.md` for syntax details. Analyzer support is planned.

### Middleware

Attach a middleware to a protocol via the `ProtocolBuilder`. Add `htmstd = "0.8"` to your `Cargo.toml` for the bundled middleware library:

```rust
use htmstd::CookieSession;

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3003")
        .single_protocol(
            ProtocolBuilder::new(HTTP::server(HttpSafety::default()))
                .append_middleware::<CookieSession>(),
        )
        .build()
);
```

`CookieSession` writes encrypted session cookies. By default, those cookies are
production-safe (`Secure`, `HttpOnly`, `SameSite=Lax`, `Path=/`). If you are
running a plain-HTTP development environment, configure the cookie safety policy
explicitly through the app config:

```rust
use htmstd::{CookieSecurity, CookieSession, CookieSessionSettings};

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3003")
        .mode(RunMode::Development)
        .set_config(CookieSessionSettings::new().security(CookieSecurity::Auto))
        .single_protocol(
            ProtocolBuilder::new(HTTP::server(HttpSafety::default()))
                .append_middleware::<CookieSession>(),
        )
        .build()
);
```

`CookieSecurity::Auto` follows `RunMode`: `Production`/`Beta` keep `Secure`
cookies, while `Development`/`Build` allow plain HTTP cookies. For production,
also configure a stable `SessionSecret` so sessions survive process restarts.

Middleware can also be attached per-endpoint via `middleware = [...]` inside the `endpoint!` block — see `example_hotaru` for the pattern.

### Templates

Render HTML with Akari via `akari_render!` — the macro looks up the template file and substitutes the named bindings:

```rust
endpoint! {
    APP.url("/profile"),
    pub profile<HTTP> {
        akari_render!("profile.html", name = "Alice")
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

## Examples

Check out the [example repository](https://github.com/Field-of-Dreams-Studio/hotaru-example) for:
- Basic routing and handlers
- Form processing and file uploads
- Session management with cookies
- CORS configuration
- Multi-protocol applications

## Crate Ecosystem

Hotaru is built on a modular architecture:

- **[hotaru](https://crates.io/crates/hotaru)** - Main framework with convenient API
- **[hotaru_core](https://crates.io/crates/hotaru_core)** - Core protocol and routing engine
- **[hotaru_trans](https://crates.io/crates/hotaru_trans)** - Procedural macros for endpoint! and middleware!
- **[hotaru_http](https://crates.io/crates/hotaru_http)** - HTTP implementation for Hotaru
- **[hotaru_tls](https://crates.io/crates/hotaru_tls)** - TLS/HTTPS implementation for Hotaru
- **[hotaru_rt_tokio](https://crates.io/crates/hotaru_rt_tokio)** - Tokio runtime backend (`TokioRuntime`)
- **[hotaru_io_tokio](https://crates.io/crates/hotaru_io_tokio)** - Tokio TCP/IO backend (`TcpTransport`, `TokioIo`)
- **[hotaru_io_futures](https://crates.io/crates/hotaru_io_futures)** - `futures-io` adapter backend (`FuturesIo`, experimental)
- **hotaru_io_embedded** - `embedded-io-async` adapter backend (`EmbeddedIo`) — *experimental; in-workspace, unpublished (crates.io), and re-exported by `hotaru` when `io_embedded` is enabled*
- **[hotaru_lib](https://crates.io/crates/hotaru_lib)** - Utility functions (compression, encoding, etc.)
- **[htmstd](https://crates.io/crates/htmstd)** - Standard middleware library (CORS, sessions)

## Changelog

### 0.8.4 (Current)
- Continued backend split work by moving Tokio-specific IO/runtime support out of `hotaru_core`.
- Clarified platform and task-mobility feature modes.
- Added explicit local-executor refinements: `spawn_local_atomic` and `spawn_local_no_atomic`.
- Made sync primitive selection feature-based: `parking_lot`, `spin`, or Hotaru `RefCell` fallback.
- Removed hidden `target_has_atomic` behavior from core feature selection.
- Replaced the old `full`/`lite` regex names with additive `full_regex` / `lite_regex`; omitting both uses Hotaru's no-regex stub path.
- Improved `hotaru` facade dependency gating and reduced default-feature leakage.
- Continued preparation for a smaller backend-neutral core.

### 0.8.3
- **Core/backend split**: `hotaru_core` is now backend-neutral at the public type layer. Concrete Tokio runtime and TCP/IO implementations moved into sibling crates (`hotaru_rt_tokio`, `hotaru_io_tokio`), while the umbrella `hotaru` crate keeps the familiar Tokio defaults.
- **IO adapter crates**: futures-io and embedded-io-async adapters moved out of core into `hotaru_io_futures` and `hotaru_io_embedded`. Each backend uses local wrapper types (`TokioIo<T>`, `FuturesIo<T>`, `EmbeddedIo<T>`) so adapter impls stay additive and avoid trait-coherence conflicts.
- **Simpler `hotaru_core` features**: core no longer owns `io_*`, `rt_*`, `tokio`, or `embassy` feature flags. It now keeps only the platform axis (`std` / `embedded`) and task-mobility axis (`spawn_send` / `spawn_local`); runtime and IO backends are selected through backend crates, or through optional facade features on `hotaru`.
- **`hotaru` facade defaults to Tokio/std**: the umbrella keeps Tokio as the supported default path, while exposing experimental optional `embedded`, `embassy`, and `io_embedded` features for in-workspace backend work. `io_embedded` re-exports `EmbeddedIo`; the backend crate remains unpublished on crates.io.
- **Runtime abstraction cleanup**: `RuntimeSpec` is the backend-neutral runtime trait, with Tokio implemented externally by `hotaru_rt_tokio::TokioRuntime`. Framework types (`Server`, `Client`, builders, and URL/protocol-entry types) now carry explicit transport/runtime parameters in core, while `hotaru` restores ergonomic defaults.
- **`MaybeSend` task-mobility model**: async framework surfaces use `MaybeSend` so `spawn_send` builds keep real `Send` bounds and `spawn_local` builds can support local `!Send` futures. `hotaru_io_embedded` gates its actual embedded-io-async trait impls on `spawn_local`, not on the `embedded` platform flag.
- **Framework-owned async IO traits**: `HotaruRead`, `HotaruWrite`, `HotaruBufRead`, `HotaruBufWrite`, `HotaruIOError`, `HotaruBufReader`, and `HotaruBufWriter` provide the common IO trait surface used by transports and protocols without hardcoding Tokio types in core.
- **Native async trait surfaces**: core transport/protocol traits use return-position `impl Future` instead of `async-trait`, reducing proc-macro dependency surface and avoiding unnecessary boxed futures at trait boundaries.
- **Protocol-agnostic endpoint outcomes**: `EndpointOutcome<C>` lets generated endpoints apply return values to any request context. HTTP keeps the existing `HttpResponse` endpoint style, while non-HTTP/inbound-only protocols can use `()` outcomes without placeholder responses.
- **Per-protocol URL parsing hooks**: `Protocol` can customize URL tokenization/literal parsing, and URL parser internals such as `RawToken`, `TypeKind`, `tokenize`, and `tokens_to_patterns` are re-exported for protocol-specific routing work.
- **Preferred-language middleware**: `htmstd` adds `PreferredLanguageMiddleware`, `PreferredLanguage`, settings, and request-extension helpers for parsing and negotiating the `Accept-Language` header.
- **no_std preparation**: core continues moving toward `no_std` readiness with `alloc` usage, `core` imports, Akari `embedded`/`no_std` alignment, generic IO errors, and backend-neutral abstractions. Embassy and embedded backend work exists in-tree but remains experimental.
- **Sync-main entry macros**: `run_server!` / `run_server_until!` (blocking) and `run_server_no_block!` / `run_server_no_block_until!` (fire-and-forget) let users run a server from an ordinary `fn main()` — no `#[tokio::main]`, no `async fn main`. Backed by a new `BlockingRuntimeCap` capability trait implemented by `TokioRuntime`.

### 0.8.2
- `http` and `http_compression` moved to optional features (compression default-off)
- HTTP re-exports relocated to `hotaru::http`
- Clean builds ~35% faster (dropped `tracing`, gated heavy codecs)
- `regex` bumped 1.5.6 -> 1.12
- `AccessPointTable` switched to `PRwLock` (no more poisoning)
- `hotaru_trans` `..` middleware inheritance now honors the URL's app ident
- `hotaru_trans` anonymous-fn `_` form fixed 
- Client / outpoint runtime paired with `Client<TS>`, the `outpoint!` macro, and `run!` / `call!` invocation sugar
- Protocol trait reshape: channel-based `open_channel` / `handle` / `send`; new `Channel` trait + `ProtocolFlow`
- `RequestContext` rework: `Default` supertrait, `type Channel` anchor, `inject_request` / `into_response`; new `EmptyError`
- Result-typed execution chain — no boxing at chain boundaries
- Named access points with a single canonical registration funnel
- Instance-based transports: `TransportSpec::Inbound` / `Outbound` replace `Accepter` / `Connector`
- HTTPS feature: `HTTPS = Http1Protocol<TlsStream, TlsTransport>`
- New `LServer!` / `LClient!` / `LUrl!` / `LPattern!` macros replace `LApp!`

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

## Learn More

- **Akari Template Engine**: https://crates.io/crates/akari
- **Homepage**: https://hotaru.rs
- **Documentation Home Page**: https://fds.rs
- **GitHub**: https://github.com/Field-of-Dreams-Studio/hotaru
- **Documentation**: https://docs.rs/hotaru

| Video Resources | URL |
| --- | --- |
| Quick Tutorial | Youtube: https://www.youtube.com/watch?v=8pV-o04GuKk&t=6s <br> Bilibili: https://www.bilibili.com/video/BV1BamFB7E8n/ |

## AI-assisted development

AI-assistance tiers describe the kind of collaboration, not a percentage of
generated code.

| Tier | Rule |
| --- | --- |
| **Forbidden** | Design, proofs, semantics, and novel logic are human-authored. |
| **Author-Owned** | AI may assist with drafts or completion; the human owns the design and committed work. |
| **Human-Led** | The human writes the structure and load-bearing logic; AI may assist with helpers and boilerplate. |
| **Co-Authored** | AI may assist with design and implementation; the human must fully internalize the result. |

In every tier, contributors must understand, explain, modify, and debug their
work. AI may assist with tests, documentation, and mechanical typing after the
relevant human decisions are settled. Reviewers may request an explanation or
walkthrough.

Per-component declarations are listed in
[GOVERNANCE.md](https://github.com/Field-of-Dreams-Studio/hotaru/blob/main/GOVERNANCE.md#component-ownership).

## 📄 License

MIT License — see [LICENSE.txt](LICENSE.txt).

Copyright (c) 2024-2026 @ [Field of Dreams Studio (FDS)](https://fds.moe) & [Project-StarFall](https://sf.fds.moe) & [PMINE-FDS](https://pmine.rs)
