# htmstd - Hotaru Standard Middleware Library

[![Crates.io](https://img.shields.io/crates/v/htmstd)](https://crates.io/crates/htmstd)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Standard middleware collection for the Hotaru 0.8.x framework: CORS, cookie-based sessions, request logging.

## Install

```toml
[dependencies]
hotaru = "0.8.2"
htmstd  = "0.8.2"
```

## Available middleware

Re-exported at the crate root:

- `htmstd::Cors` — CORS preflight + response-header injection. Reads `AppCorsSettings` from per-endpoint or per-protocol config.
- `htmstd::CookieSession`, `htmstd::Session` — encrypted cookie-backed sessions.
- `htmstd::PrintLog` — minimal request logger.
- `htmstd::PreferredLanguageMiddleware`, `htmstd::PreferredLanguage` — parses `Accept-Language` and stores typed language preferences in request params.
- `htmstd::cors_settings::AppCorsSettings` — CORS policy struct.

## Attaching middleware

Middleware attaches to a `ProtocolBuilder` via `.append_middleware::<M>()`. Sessions example:

```rust
use hotaru::prelude::*;
use hotaru::http::*;
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

Reading session data from a handler:

```rust
endpoint! {
    APP.url("/login"),
    pub fn login<HTTP>(req) {
        let mut session = req.get_session();
        session.set("user_id", "12345");
        text_response("Logged in")
    }
}
```

## Preferred language

Attach `PreferredLanguageMiddleware` before handlers that need language-aware rendering:

```rust
use hotaru::prelude::*;
use hotaru::http::*;
use htmstd::{PreferredLanguageMiddleware, PreferredLanguageRequestExt};

LServer!(
    APP = Server::new()
        .binding("127.0.0.1:3003")
        .single_protocol(
            ProtocolBuilder::new(HTTP::server(HttpSafety::default()))
                .append_middleware::<PreferredLanguageMiddleware>(),
        )
        .build()
);

endpoint! {
    APP.url("/hello"),
    pub fn hello<HTTP>(req) {
        let lang = req
            .preferred_language()
            .and_then(|pref| pref.best_match(["en", "zh-CN", "ja"].iter().copied()))
            .unwrap_or("en");

        text_response(format!("selected language: {lang}"))
    }
}
```

Without the extension trait, downstream code can also read `req.params.get::<PreferredLanguage>()` directly.

## CORS

Configure CORS per protocol (global) by appending the `Cors` middleware and supplying an `AppCorsSettings` in `config`:

```rust
use hotaru::prelude::*;
use hotaru::http::*;
use htmstd::{Cors, cors_settings::AppCorsSettings};

endpoint! {
    APP.url("/api/data"),
    config = [
        AppCorsSettings::default()
            // refine as needed; see AppCorsSettings fields
    ],
    middleware = [Cors],
    pub fn data<HTTP>(req) {
        text_response("ok")
    }
}
```

## HTTP safety per endpoint

`HttpSafety` lives in `hotaru::http::HttpSafety` (formerly `hotaru_core::http::*` in 0.7-era code). Configure per endpoint:

```rust
use hotaru::http::{HttpSafety, HttpMethod};

endpoint! {
    APP.url("/api/upload"),
    config = [
        HttpSafety::new()
            .with_max_body_size(50 * 1024 * 1024) // 50 MB
            .with_allowed_methods(vec![HttpMethod::POST])
    ],
    pub fn upload<HTTP>(req) {
        // ...
    }
}
```

## Examples

For complete examples, see `example_hotaru` in the workspace.

## License

MIT

## Part of Hotaru Framework

Standard middleware library for the [Hotaru web framework](https://crates.io/crates/hotaru). Learn more: https://hotaru.rs
