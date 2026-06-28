The Hotaru 0.8 era starts from 23/May/2026.

# Hotaru Web Framework

![Latest Version](https://img.shields.io/badge/version-0.8.2-brightgreen)
[![Crates.io](https://img.shields.io/crates/v/hotaru)](https://crates.io/crates/hotaru)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE.txt)

> Small, sweet, easy framework for full-stack Rust web applications 

## Overview

Hotaru is a lightweight, intuitive web framework focused on simplicity and productivity. It supports regex-based routing, tree-structured URLs, and integrates seamlessly with the Akari templating system.

The name 'Hotaru' comes from the Japanese Character '蛍' represents the firefly. 

**[Official Website](https://hotaru.rs)**

**[Example Project](https://github.com/Field-of-Dreams-Studio/hotaru-example)**

MSRV: 1.86

## Key Features

- **Multi-Protocol Support**: Handle HTTP/HTTPS, WebSocket, and custom TCP protocols 
- **Simple API**: Intuitive request/response handling with minimal boilerplate
- **Full-Stack**: Built-in template rendering with Akari templates
- **Flexible Routing**: Support for regex patterns, literal URLs, and nested routes
- **Asynchronous**: Built with Tokio for efficient async handling
- **Form Handling**: Easy processing of form data and file uploads
- **Middleware Support**: Create reusable request processing chains

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
hotaru = "0.8.2"
tokio = { version = "1", features = ["full"] }
```

### Optional Features

Default features: `trans`, `http`. Cargo's additive feature unification means sub-features pull in their prerequisites automatically — you never have to enable a base feature by hand.

**Protocol stack**

- **`http`** *(default-on)*: HTTP/1.1 stack (`hotaru_http` + `ahttpm`). Opt out with `default-features = false` for protocol-only builds (e.g. gRPC-only deployments) — `hotaru::http::*`, `HTTP`, `HttpContext`, `HttpRequest`, `HttpResponse`, etc. then disappear from the crate surface.
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
  hotaru = { version = "0.8.2", features = ["external-ctor"] }
  ctor = "0.4.0"
  tokio = { version = "1", features = ["full"] }
  ```

**Example — HTTPS server with body compression:**

```toml
[dependencies]
hotaru = { version = "0.8.2", features = ["https", "http_compression"] }
tokio = { version = "1", features = ["full"] }
```

**Example — gRPC-only (no HTTP):**

```toml
[dependencies]
hotaru = { version = "0.8.2", default-features = false, features = ["trans"] }
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
- Remaining readme examples use `trans`. To switch, set `default-features = false` on the `hotaru` dependency and turn on the flavor you want, e.g. `hotaru = { version = "0.8.2", default-features = false, features = ["semi-trans", "http"] }`. Cargo feature unification would otherwise keep `trans` on alongside it; remember to re-add `http` since `default-features = false` also drops the default HTTP stack.
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
- **[hotaru_lib](https://crates.io/crates/hotaru_lib)** - Utility functions (compression, encoding, etc.)
- **[htmstd](https://crates.io/crates/htmstd)** - Standard middleware library (CORS, sessions)

## Changelog

### 0.8.3 (Current)
- **Protocol-agnostic `endpoint!` via `EndpointOutcome`**: new `EndpointOutcome<C>` trait in `hotaru_core::protocol` (re-exported from `hotaru::prelude`) decouples endpoint return values from the HTTP `response` field. Generated handlers now return `impl EndpointOutcome<Ctx> + 'static`; the wrapper applies the outcome via `EndpointOutcome::apply_to(__outcome, &mut req)?` instead of writing `req.response` directly. Generic impls for `()` (no-op) and `Result<O, C::Error>` (fallible bodies) live in `hotaru_core`; `impl EndpointOutcome<HttpContext<TS>> for HttpResponse` lives in `hotaru_http`. **Existing HTTP endpoint bodies compile unchanged** — bodies that end in `HttpResponse` still land in `ctx.response`. Inbound-only protocols can now use `()`-returning endpoints with no placeholder response.
- **Per-protocol URL parsing on `Protocol` trait**: new `tokenize_url` (pattern side, default = the framework lexer) and `lit_parser` (literal side, minimal default — HTTP overrides with `/`-split that mirrors `UrlRoot::walk_str` empty-input semantics). `lexer::tokenize` is now fallible (`Result<Vec<RawToken>, PatternError>`); `RawToken`, `TypeKind`, `tokenize`, `tokens_to_patterns` are re-exported from `hotaru_core::url`. `url::parser::parse` signature unchanged.
- **Preferred-language middleware in `htmstd`**: new `language` module exposing `PreferredLanguageMiddleware` plus the `PreferredLanguage` struct that parses the request `Accept-Language` header into ordered, q-weighted `LanguageRange` entries and stores it in `req.params`. `PreferredLanguage` provides downstream helpers — `preferred()`, `primary()`, `accepts()`, `quality_for()` / `quality_millis_for()` (q-values as `u16` milli-units), and `best_match()` / `best_match_owned()` for negotiating against a supported-language set. Configurable via `PreferredLanguageSettings` (fallback language, etc.) and ergonomic access via the `PreferredLanguageRequestExt` extension trait. All re-exported from `htmstd`.
- **Framework-owned async IO trait family (`connection::io`)**: HTTP/transport code no longer hardcodes `tokio::io` traits. New `HotaruRead` / `HotaruWrite` / `HotaruBufRead` / `HotaruBufWrite` traits, a concrete `HotaruIOError`, fallback `HotaruBufReader` / `HotaruBufWriter`, and per-backend buffered halves (`HotaruRead::Buffered` / `HotaruWrite::Buffered`, surfaced as `BufferedReadHalf<TS>` / `BufferedWriteHalf<TS>`). Tokio and `embedded-io-async` are bridged via feature-gated blanket impls, so existing Tokio transports keep working unchanged. `ConnStream` now builds on these traits and returns `Option<SocketAddr>` for `peer_addr` / `local_addr`.
- **`async-trait` dropped for native RPIT**: core async traits (`Protocol`, transport `Inbound` / `Outbound`, `HttpChannel`, etc.) now use return-position `impl Future` instead of the `async-trait` crate, removing a proc-macro dependency and per-call boxing from the trait surface.
- **no_std / `alloc` plumbing + target-flavour features**: `hotaru_core` is being prepared for `no_std`/embedded targets — `extern crate alloc`, `core::`/`alloc::` imports replacing `std::`, `akari::hash::HashMap`, and associated-type IO errors. New mutually-exclusive `std` (pulls `parking_lot`) and `embedded` (pulls `spin` + `embedded-io-async`) target markers, plus a `tokio` feature (default-on, auto-enables `std`) that gates the Tokio IO blanket impls. `lite` trims the Unicode/regex footprint for small devices.
- **Akari 0.2.8 alignment**: `hotaru_core` depends on `akari` with `default-features = false` (`dynamic`, `extension`, `object_macro`), with `full` / `lite` features mapping to `akari/full` / `akari/no_std` so embedded builds drop the heavy Unicode tables.
- **(In progress) `RuntimeSpec` runtime abstraction**: new `hotaru_core::app::runtime` module introducing a `RuntimeSpec` backend trait (spawn / time / `OnceCell` / async `Mutex` / `select2`) with a working `TokioRuntime` impl and a typecheck-only `EmbassyRuntime` stub, laying groundwork for making the Tokio dependency optional on embedded targets.

### 0.8.2
- **`http` (default-on) + `http_compression` (default-off) features**: HTTP and codecs are now optional; `default-features = false` drops HTTP entirely, `https`/`http_compression` imply `http`.
- **HTTP re-exports moved to `hotaru::http`** (`use hotaru::http::*;`); clean builds ~35 % faster (20.5 s → 13.3 s) from dropping `tracing` and gating heavy codecs.
- **Workspace + dep alignment**: five core crates pinned to 0.8.2; `regex` bumped 1.5.6 → 1.12.
- **`hotaru_core` access-point table no longer poisons**: `AccessPointTable` switched to `parking_lot::RwLock` via the existing `PRwLock` alias.
- **`hotaru_trans` `..` middleware inheritance honors the URL's app ident**: was hardcoded to `APP`, breaking multi-app setups.
- **`hotaru_trans` anonymous-fn `_` form actually works**: `_` was matched as `Punct` (it's an `Ident`), so the auto-name branch never fired.

### 0.8.x
- **Client / outpoint runtime**: new `Client<TS>` for outbound traffic, mirroring `Server<TS>`. Paired with the `outpoint!` macro (client-side counterpart to `endpoint!`) and `run!` / `call!` invocation-style sugar for one-shot and persistent outbound calls.
- **Protocol trait reshape**: channel-based `open_channel` / `handle` / `send`, plus per-protocol `install_channel` bridge. `type Context: RequestContext<Channel = Self::Channel>` pins channel-type alignment. New `Channel` trait + `ProtocolFlow` give per-exchange close semantics (HTTP/1 closes the connection; multiplexed protocols may close only the stream).
- **RequestContext rework**: `Default` supertrait; `type Channel` as a type anchor; `inject_request` / `into_response` added. `type Error: ProtocolError` is the single source of truth for chain errors, and now also requires `From<std::io::Error>` (since the client path surfaces transport-level I/O). Use the new `EmptyError` for prototypes / no-payload protocols.
- **Result-typed execution chain**: middleware, final handlers, `ExecutionChain::run`, and `UrlNode::run` all return `Result<C, <C as RequestContext>::Error>` — no boxing at the chain boundary.
- **Named access points**: every registered endpoint/outpoint has an explicit name. `ProtocolEntry::register(name, path, step_names, binding, config)` is the single canonical funnel; `Server::url(url, name, ...)` / `Client::url(url, name, ...)` are URL-first.
- **Instance-based transports**: `TransportSpec::Inbound` / `Outbound` replace the old `Accepter` / `Connector` shape. `TcpInbound::bind(target)` and `TcpOutbound::build(target)` materialize once per `Server` / `Client`.
- **HTTPS feature**: new `https` cargo feature on the umbrella `hotaru` crate forwards to `hotaru_http/tls`, surfacing `HTTPS`, `TlsTransport`, `TlsOutboundTarget`, `TlsClientConfig`. `HTTPS = Http1Protocol<TlsStream, TlsTransport>`.
- **`LServer!` / `LClient!` / `LUrl!` / `LPattern!` macros**: replace the old `LApp!` for simplified lazy-static declarations. Default to `TcpTransport`.

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

## AI Declaration of each Mod

We believe in transparency about AI-assisted development. The framework is governed jointly by two maintainer groups using a shared four-tier system that prioritizes understanding over line counts.

<details> 
<summary><b>Click for more details</b></summary> 

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
</details> 

## 📄 License

MIT License — see [LICENSE.txt](LICENSE.txt).

Copyright (c) 2024-2026 @ [Field of Dreams Studio (FDS)](https://fds.moe) & [Project-StarFall](https://sf.fds.moe) & [PMINE-FDS](https://pmine.fds.moe) 
