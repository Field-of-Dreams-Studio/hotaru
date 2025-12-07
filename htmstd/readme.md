# htmstd - Hotaru Standard Middleware Library

[![Crates.io](https://img.shields.io/crates/v/htmstd)](https://crates.io/crates/htmstd)
[![GPL-3.0 License](https://img.shields.io/badge/license-GNU-blue.svg)](LICENSE)

Standard middleware collection for the Hotaru web framework, providing common functionality like CORS, sessions, and authentication.

## Middleware

### CORS (Cross-Origin Resource Sharing)

Configure CORS policies for your application:

```rust
use htmstd::cors::AppCorsSettings;

let app = App::new()
    .set_config(
        AppCorsSettings::new()
            .allow_origin("https://example.com")
            .allow_methods(vec!["GET", "POST"])
    )
    .build();
```

### Cookie-Based Sessions

Secure session management using encrypted cookies:

```rust
use htmstd::session::CookieSession;

let app = App::new()
    .append_middleware::<CookieSession>()
    .build();
```

Access session data in handlers:

```rust
endpoint! {
    APP.url("/login"),
    pub login<HTTP> {
        let mut session = ctx.get_session();
        session.set("user_id", "12345");
        text_response("Logged in")
    }
}
```

## Safety Configuration

Configure HTTP safety limits per endpoint or globally:

```rust
use hotaru_core::http::HttpSafety;
use hotaru_core::http::HttpMethod;

endpoint! {
    APP.url("/api/upload"),
    config=[HttpSafety::new()
        .with_max_body_size(50 * 1024 * 1024)  // 50MB
        .with_allowed_methods(vec![HttpMethod::POST])
    ],
    pub upload<HTTP> {
        // Handle file upload
    }
}
```

## Examples

For complete examples, see:
- [Session Example](https://github.com/Field-of-Dreams-Studio/sfx)
- [CORS Example](https://github.com/Field-of-Dreams-Studio/hotaru-example)

## License

GPL-3.0 License

## Part of Hotaru Framework

This is the standard middleware library for the [Hotaru web framework](https://crates.io/crates/hotaru).

Learn more: https://fds.rs
