# hotaru_meta

[![Crates.io](https://img.shields.io/crates/v/hotaru_meta)](https://crates.io/crates/hotaru_meta)

Procedural macros for the Hotaru web framework, providing declarative syntax for endpoint and middleware definition.

## Macros

### `endpoint!`

Define HTTP endpoints with automatic registration:

```rust
use hotaru_meta::endpoint;

endpoint! {
    APP.url("/api/users"),
    pub handle_users<HTTP> {
        text_response("User list")
    }
}
```

### `middleware!`

Define middleware with clean syntax:

```rust
use hotaru_meta::middleware;

middleware! {
    pub AuthMiddleware<HTTP> {
        // Middleware implementation
    }
}
```

## Features

- Declarative endpoint registration
- Protocol-generic middleware definition
- Compile-time route validation
- Automatic handler wrapping
- Zero runtime overhead

## Usage

This crate is automatically included when using the Hotaru framework. You typically don't need to add it directly.

## License

GPL-3.0 License

## Part of Hotaru Framework

This is the macro crate for the [Hotaru web framework](https://crates.io/crates/hotaru).

Learn more: https://fds.rs
