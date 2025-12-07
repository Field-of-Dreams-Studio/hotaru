# Full Hyper API Access in h2per

h2per provides complete access to Hyper's APIs while maintaining compatibility with Hotaru's handler system. This document explains how to access all Hyper functionality.

## Overview

h2per wraps Hyper's types but provides direct access through:
1. **Public `inner` fields** - Direct access to underlying Hyper types
2. **Re-exported types** - All commonly used Hyper types are re-exported
3. **Pass-through methods** - Convenience methods that delegate to Hyper
4. **Full control methods** - Methods to get/set the inner Hyper objects

## Accessing Hyper Types

### 1. Direct Access via Inner Fields

```rust
endpoint! {
    APP.url("/example"),
    pub example <HYPER1> {
        // Direct access to Hyper's Request
        let hyper_request: &Request<Body> = &req.request().inner;
        
        // Direct access to Hyper's Response  
        let hyper_response: &mut Response<Body> = &mut req.response_mut().inner;
        
        // Now you can use ANY Hyper API
        let headers = hyper_request.headers();
        let method = hyper_request.method();
        let uri = hyper_request.uri();
    }
}
```

### 2. Using Helper Methods

```rust
// Get references to inner Hyper types
let request = req.request().as_inner();      // &Request<Body>
let response = req.response_mut().as_inner_mut(); // &mut Response<Body>

// Take ownership of inner types
let owned_request = req.request().into_inner();   // Request<Body>
let owned_response = req.response().into_inner();  // Response<Body>
```

### 3. Accessing Headers

```rust
// Get request headers (returns &HeaderMap)
let headers = req.request().headers();

// Get mutable request headers
let headers_mut = req.request_mut().headers_mut();

// Get response headers
let response_headers = req.response().headers();

// Get mutable response headers
let response_headers_mut = req.response_mut().headers_mut();

// Use Hyper's header constants
use h2per::hyper_exports::*;
headers.get(USER_AGENT);
headers.get(CONTENT_TYPE);
headers.get(AUTHORIZATION);
```

### 4. Working with Body

```rust
use h2per::hyper_exports::body;

// Create different body types
let empty = body::empty();
let from_bytes = body::from_bytes(b"Hello");
let from_string = body::from_string("Hello".to_string());
let from_vec = body::from_vec(vec![1, 2, 3]);

// Set streaming body
req.response_mut().set_body_stream(custom_body);

// Access body utilities
use h2per::hyper_exports::{BodyExt, Full, Empty};
```

### 5. Using Extensions

```rust
// Store custom data in request
#[derive(Clone)]
struct RequestId(String);

req.request_mut().extensions_mut().insert(RequestId("123".into()));

// Retrieve custom data
let id = req.request().extensions().get::<RequestId>();

// Same for response
req.response_mut().extensions_mut().insert(MyData::new());
```

### 6. HTTP Version and Protocol Info

```rust
use h2per::hyper_exports::Version;

// Get version
let version = req.request().version();
match version {
    Version::HTTP_09 => {},
    Version::HTTP_10 => {},
    Version::HTTP_11 => {},
    Version::HTTP_2 => {},
    Version::HTTP_3 => {},
    _ => {},
}

// Set version
req.response_mut().set_version(Version::HTTP_2);
```

### 7. URI Components

```rust
let uri = req.request().uri();

// Access all URI components
let scheme = uri.scheme_str();
let authority = uri.authority();
let path = uri.path();
let query = uri.query();
let host = uri.host();
let port = uri.port_u16();
```

### 8. Status Codes

```rust
use h2per::hyper_exports::StatusCode;

// Get status
let status = req.response().status();

// Set status using Hyper's StatusCode
req.response_mut().set_status(StatusCode::CREATED);
req.response_mut().set_status(StatusCode::NOT_MODIFIED);
req.response_mut().set_status(StatusCode::INTERNAL_SERVER_ERROR);
```

## Complete API Access

### Request Methods Available

Through `req.request()` you can access:
- `inner: Request<Body>` - Direct Hyper Request access
- `method()` - Get HTTP method
- `uri()` - Get full URI
- `version()` - Get HTTP version  
- `headers()` - Get headers
- `headers_mut()` - Modify headers
- `extensions()` - Get extensions
- `extensions_mut()` - Modify extensions
- `as_inner()` - Get &Request<Body>
- `as_inner_mut()` - Get &mut Request<Body>
- `into_inner()` - Take ownership of Request<Body>

### Response Methods Available

Through `req.response_mut()` you can access:
- `inner: Response<Body>` - Direct Hyper Response access
- `status()` - Get status code
- `set_status()` - Set status code
- `version()` - Get HTTP version
- `set_version()` - Set HTTP version
- `headers()` - Get headers
- `headers_mut()` - Modify headers
- `extensions()` - Get extensions
- `extensions_mut()` - Modify extensions
- `set_body()` - Set body from bytes
- `set_body_bytes()` - Set body from Bytes
- `set_body_stream()` - Set streaming body
- `as_inner()` - Get &Response<Body>
- `as_inner_mut()` - Get &mut Response<Body>
- `into_inner()` - Take ownership of Response<Body>

## Imported Hyper Modules

The `h2per::hyper_exports` module re-exports:

### Core Types
- `Method`, `StatusCode`, `Version`, `Uri`
- `Request`, `Response` 
- `HeaderMap`, `HeaderName`, `HeaderValue`
- `Extensions`
- `Bytes`, `BytesMut`

### All Header Constants
- `ACCEPT`, `AUTHORIZATION`, `CONTENT_TYPE`, `USER_AGENT`
- `CACHE_CONTROL`, `SET_COOKIE`, `COOKIE`
- And 70+ more standard headers

### Body Utilities
- `BodyExt` - Extension trait for bodies
- `Full` - Complete body
- `Empty` - Empty body
- `BoxBody` - Type-erased body

### Connection Builders
- `hyper::server::conn::{http1, http2}`
- `hyper::client::conn::{http1, http2}`

### Service Trait
- `hyper::service::Service` - For custom services

## Example: Using All Features

```rust
use h2per::hyper_exports::*;

endpoint! {
    APP.url("/advanced"),
    pub advanced <HYPER1> {
        // 1. Direct header access
        let auth = req.request().headers().get(AUTHORIZATION);
        
        // 2. Modify response headers
        req.response_mut().headers_mut().insert(
            X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY")
        );
        
        // 3. Use extensions for custom data
        req.request_mut().extensions_mut().insert(MyState::new());
        
        // 4. Check HTTP version
        if req.request().version() == Version::HTTP_2 {
            // HTTP/2 specific logic
        }
        
        // 5. Build custom response
        let response = Response::builder()
            .status(StatusCode::CREATED)
            .header(LOCATION, "/resource/123")
            .body(body::from_string("Created"))
            .unwrap();
            
        // 6. Replace entire response
        req.response_mut().inner = response;
    }
}
```

## Summary

With h2per, you have **complete access** to all Hyper APIs:

1. **Every struct field** in Request/Response is accessible
2. **All methods** from Hyper are available via `.inner`
3. **All types** are re-exported for convenience
4. **No functionality is hidden** - you can always access the raw Hyper types

This means you can use any Hyper tutorial, example, or documentation directly with h2per - just access the inner types when needed!