## Design Rationale: Why Proc Macros?

---

## Why `endpoint!` and `middleware!`?

Hotaru is opinionated: an endpoint's URL, middleware, config, and handler belong together and are registered automatically at startup.

**The Problem**

Traditional Rust web frameworks scatter endpoint configuration across multiple attributes:

```rust
#[get("/users/<id>")]
#[middleware::auth]
#[middleware::rate_limit(100)]
async fn get_user(...) -> impl Responder {
    // handler code
}

// And somewhere else, you need to register it:
// router.register(get_user)
```

Configuration lives above the function. Registration happens elsewhere. To understand what an endpoint does and whether it is active, you need to check multiple places.

**Our Approach**

```rust
// Requires: use hotaru::prelude::*; use hotaru::http::*;
endpoint! {
    APP.url("/users/<int:id>"),
    middleware = [.., AuthCheck, RateLimit],
    config = [HttpSafety::new().with_allowed_methods(vec![GET, POST])],

    /// Get a user
    pub get_user <HTTP> {
        let id = req.pattern("id").unwrap_or("0".to_string());
        json_response(object!({ id: &id }))
    }
}
```

One block. One place. URL, middleware, config, handler, and automatic registration.

---

## Macro Syntax (Correct)

### Endpoint

```rust
endpoint! {
    <url expr>,
    middleware = [.., LocalMw, ...],   // optional
    config = [ParamValue, ...],        // optional

    /// Doc comment (inside the macro)
    pub handler_name <Protocol> { ... }
    // or: _ <Protocol> { ... } for anonymous endpoints
}
```

Notes:
- Use brace syntax `{}` for both `endpoint!` and `middleware!` (no parentheses).
- `middleware` and `config` are optional and must appear before the handler.
- If `middleware` is omitted, the endpoint inherits protocol-level middleware by default.
- Use `..` inside `middleware = [...]` to insert protocol-level middleware at that point.
- To opt out of global middleware, specify local middleware without `..` (at least one entry is required).
- `config` entries are stored via `Url::set_params`, so they must implement `ParamValue` (for HTTP this includes `HttpSafety`).

### Middleware

```rust
middleware! {
    /// Logs every request
    pub LogRequest <HTTP> {
        let start = std::time::Instant::now();
        let req = next(req).await;
        println!("Completed in {:?}", start.elapsed());
        req
    }
}
```

Short-circuit example:

```rust
middleware! {
    pub AuthCheck <HTTP> {
        if !is_authorized(&req) {
            req.response = text_response("Unauthorized");
            return req;
        }
        next(req).await
    }
}
```

---

## Key Features

1. **Automatic Registration**: Macros generate a constructor-based registration hook, so endpoints are wired without manual `router.register()` calls. This assumes a static `APP` (e.g., `SApp` or `LApp!`) is in scope.
2. **Readable at a Glance**: Everything about a route lives in a single block.
3. **Middleware Composition with `..`**: Control ordering with explicit inheritance markers.
4. **Per-Endpoint Config**: `config = [...]` attaches typed values like `HttpSafety` to the URL node.

### Multi-Protocol Support (Requires Registered Protocols)

You can use the same URL path with different protocols as long as you use unique handler names and have both protocols registered on the app:

```rust
endpoint! { APP.url("/chat"), pub http_chat <HTTP> { /* ... */ } }
endpoint! { APP.url("/chat"), pub custom_chat <MyProtocol> { /* ... */ } }
```

---

## Important Rules

- One handler per URL per protocol. For HTTP, use method routing inside the handler (GET/POST/PUT, etc.).
- Doc comments and attributes must be inside the macro, after the URL line and before the handler name.
- The `..` middleware inheritance token requires the `APP` static to be named `APP` and in scope.

---

## Trade-offs and Roadmap

| Limitation | Status |
|------------|--------|
| Custom syntax has a learning curve | Compiles to standard async Rust |
| Limited `rustfmt` support | Planned: custom formatter |
| IDE support varies | Optimized for rust-analyzer |

We are building toward a complete toolchain, including our own formatter and better IDE support. Hotaru is evolving from a framework into a web endpoint DSL with proper tooling.

---

## Compile-Time Guarantees

All macros expand at compile time. Zero runtime overhead. The compiler sees normal Rust after expansion.

---

### Contributing

We are a small team building something ambitious. If you are interested in:

- Language tooling (formatters, analyzers)
- Macro systems and DSL design
- Web framework internals
- Or just want to debate syntax decisions with us

We would love to hear from you. Check out our GitHub at https://github.com/Field-of-Dreams-Studio/hotaru or open an issue to start a conversation.
