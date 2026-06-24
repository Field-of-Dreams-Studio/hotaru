# htmstd - Hotaru Standard Middleware Library

[![Crates.io](https://img.shields.io/crates/v/htmstd)](https://crates.io/crates/htmstd)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Standard middleware collection for the Hotaru 0.8.x framework: CORS, cookie-based sessions, request logging.

## Install

```toml
[dependencies]
hotaru = "0.8.2"
htmstd  = "0.8.3"
```

## Available middleware

Re-exported at the crate root:

- `htmstd::Cors` — CORS preflight + response-header injection. Reads `AppCorsSettings` from per-endpoint or per-protocol config.
- `htmstd::CookieSession`, `htmstd::Session` — encrypted cookie-backed sessions.
- `htmstd::CookieSessionSettings`, `htmstd::CookieSecurity` — cookie-session safety settings.
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
use hotaru::Value;
use htmstd::session::CSessionRW;

endpoint! {
    APP.url("/login"),
    pub fn login<HTTP>(req) {
        let session = req
            .params
            .get_mut::<CSessionRW>()
            .expect("CookieSession middleware should install CSessionRW");
        session.insert("user_id".to_string(), Value::new("12345"));

        text_response("Logged in")
    }
}
```

### Cookie-session safety

`CookieSession` writes encrypted session cookies. The default cookie attributes
are production-safe:

- `Secure`
- `HttpOnly`
- `SameSite=Lax`
- `Path=/`

Browsers do not send `Secure` cookies over plain HTTP. If you run a local or
trusted plain-HTTP environment, register `CookieSessionSettings` in the app
config:

```rust
use hotaru::prelude::*;
use hotaru::http::*;
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

`CookieSecurity::Auto` resolves from the app `RunMode`:

- `Production` and `Beta` => `Secure`
- `Development` and `Build` => plain HTTP cookies

You can also opt in directly:

```rust
CookieSessionSettings::new().secure();   // always Secure
CookieSessionSettings::new().insecure(); // never Secure; local/plain HTTP only
```

For production, register a stable `SessionSecret` too; otherwise
`CookieSession` falls back to a random per-process secret and sessions are
invalidated on restart:

```rust
use htmstd::{CookieSessionSettings, SessionSecret};

Server::new()
    .set_config(SessionSecret::new("at-least-32-bytes-of-random-secret-material"))
    .set_config(CookieSessionSettings::new().secure());
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
