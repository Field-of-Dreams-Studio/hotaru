# Hotaru Programming Style Guide

This guide defines the programming conventions and best practices for developing with the Hotaru framework, extracted from analyzing the codebase examples and verified through compilation.

## 1. Philosophy & Core Concepts

### Protocol Abstraction
- **One endpoint, one protocol** - Each endpoint handles a single protocol type (specified with `<PROTOCOL>`)
- **Multi-protocol apps** - A single APP instance can serve multiple protocols simultaneously
- **Write once per protocol** - Create protocol-specific endpoints, deploy on multi-protocol servers

### Core Design Principles
- **Macro-Driven Development** - Leverage macros for endpoints and middleware
- **Zero-Cost Abstractions** - Performance without compromise
- **Type-First Design** - Explicit type annotations for clarity and safety
- **Automatic Async** - Hotaru automatically makes all endpoints and middleware async
- **Request Context Architecture** - Endpoints receive request contexts (e.g., HttpContext), not raw protocol messages or instances

## 2. Project Setup & Structure

### Initialization
```bash
# Start a new Hotaru project
hotaru init

# Create new components
hotaru new [component]
```

### Dependencies
**Important:** Always use `ctor = "0.4"` in your Cargo.toml:
```toml
[dependencies]
ctor = "0.4"
hotaru = "*"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### Directory Structure
```
project/
├── src/
│   ├── main.rs           # App definition and main entry
│   ├── endpoints/        # Endpoint modules (optional organization)
│   └── middleware/       # Middleware modules (optional organization)
├── templates/            # HTTP template files (Akari templates)
├── programfiles/         # Configuration and runtime data (auto-copy enabled)
└── Cargo.toml
```

### Import Conventions
```rust
// Always use prelude for common types
use hotaru::prelude::*;

// Protocol-specific imports
use hotaru::http::*;

// External dependencies after Hotaru imports
use serde_json::json;
use tokio::time::sleep;
```

## 3. Application Patterns

### Static APP Pattern
```rust
use hotaru::prelude::*;
use hotaru::http::*;

// Define app as static with lazy initialization
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3003")
        .build()
});

#[tokio::main]
async fn main() {
    APP.clone().run().await;
}
```

### Multi-Protocol Setup
```rust
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3003")
        .handle(
            HandlerBuilder::new()
                .protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
                .protocol(ProtocolBuilder::new(TcpProtocol::new(ProtocolRole::Server)))
                .protocol(ProtocolBuilder::new(HYPER1::new(ProtocolRole::Server)))
        )
        .build()
});
```

## 4. Macro Syntax Convention

### Always Use Brace Syntax {} for Macros

**Important:** Always use brace syntax `{}` for `endpoint!` and `middleware!` macros, never parenthesis syntax `()`.

#### ✅ Correct (Brace Syntax)
```rust
endpoint! {
    APP.url("/users"),
    pub get_users <HTTP> {
        // No semicolon needed after closing brace
        json_response(fetch_users())
    }
}

middleware! {
    pub Logger <HTTP> {
        println!("Request: {}", req.path());
        next(req).await
    }
}
```

#### Optional fn-style (request naming)
```rust
endpoint! {
    APP.url("/users"),
    pub fn get_users(req: HTTP) {
        json_response(fetch_users())
    }
}

middleware! {
    pub fn Logger(req: HTTP) {
        println!("Request: {}", req.path());
        next(req).await
    }
}
```

#### ❌ Incorrect (Parenthesis Syntax)
```rust
// DO NOT USE THIS STYLE
endpoint!(
    APP.url("/users"),
    pub get_users <HTTP> {
        json_response(fetch_users())
    }
);  // Requires semicolon - easy to forget!

middleware!(
    pub Logger <HTTP> {
        println!("Request: {}", req.path());
        next(req).await
    }
);  // Another semicolon to remember
```

### Why Brace Syntax?

1. **Semantic Correctness** - Braces indicate definitions (like `struct {}`, `impl {}`), while parentheses indicate function calls
2. **No Semicolons** - Brace syntax doesn't require trailing semicolons, reducing errors
3. **Better Error Messages** - Missing closing brace gives clearer "unclosed delimiter" error
4. **Visual Clarity** - Creates clear boundaries for multi-line definitions
5. **Rust Idioms** - Follows Rust's pattern of using braces for definitions
6. **Editor Support** - Better code folding and navigation

This is a **mandatory style rule** for all Hotaru code.

## 5. Endpoint Development

### Request Context (Critical Concept)
**Important:** Endpoints receive a request context object, NOT raw protocol messages or protocol instances.

For HTTP endpoints, the `req` variable is an `HttpContext` that provides:
- Parsed request data (method, path, headers, body)
- Response building utilities
- Pattern matching for URL parameters
- Access to middleware-set values

```rust
endpoint! {
    APP.url("/example/<id>"),
    
    pub example_handler <HTTP> {
        // 'req' is an HttpContext, NOT:
        // - Raw HTTP bytes
        // - HttpRequest/HttpResponse objects  
        // - HTTP protocol instance
        
        // The context provides these conveniences:
        let method = req.method();           // HTTP method
        let path = req.path();               // Request path
        let id = req.pattern("id");          // Path parameters
        let headers = req.headers();         // Parsed headers
        
        text_response("Processed by context")
    }
}
```

### Basic Endpoint (Function Style)
```rust
endpoint! {
    APP.url("/path"),
    
    /// Brief description of endpoint
    pub function_name <Protocol> {
        // Implementation - automatically async
        text_response("Hello, World!")
    }
}
```

### Anonymous Endpoint
```rust
endpoint! {
    APP.url("/anonymous"),
    
    _ <Protocol> {
        // Anonymous endpoint - no function name
        text_response("Anonymous response")
    }
}
```

### URL Pattern Matching
```rust
endpoint! {
    APP.url("/<int:id>/<name>"),
    
    /// Pattern matching with typed parameters
    pub user_profile <HTTP> {
        let id = req.pattern("id").unwrap_or("0".to_string());
        let name = req.pattern("name").unwrap_or("unknown".to_string());
        
        // Note: Use references to avoid move errors
        json_response(object!({
            user_id: &id,
            user_name: &name,
            profile_url: format!("/user/{}/profile", id)
        }))
    }
}
```

**Pattern Types:**
- `<int:param>` - Integer parameters
- `<float:param>` - Float parameters
- `<param>` - String parameters (default)

### Critical: URL Uniqueness Constraint

**Each URL can only have ONE handler function in Hotaru.** This is a fundamental architectural design.

#### Why Duplicate URLs Fail

1. **Single Handler Per URL**: Internally, each `Url<C>` struct stores exactly one handler:
```rust
pub method: RwLock<Option<Arc<dyn AsyncFinalHandler<C>>>>
```

2. **Last Registration Wins**: If you define multiple endpoints with the same URL:
```rust
// ❌ WRONG - Second endpoint overwrites the first!
endpoint! { 
    APP.url("/api/users"), 
    pub get_users <HTTP> { /* GET logic */ }
}

endpoint! { 
    APP.url("/api/users"), 
    pub create_user <HTTP> { /* POST logic */ }
}
// Only create_user will work - get_users is completely replaced!
```

3. **Compiler Errors**: Duplicate handler names cause "function defined multiple times" errors because the macro generates functions with identical names.

#### The Correct Solution: Method Routing

Handle different HTTP methods within a single endpoint:

```rust
// ✅ CORRECT - One handler per URL, route by method
endpoint! {
    APP.url("/api/users"),
    
    /// Handles all operations on the users resource
    pub users_handler <HTTP> {
        match req.method() {
            "GET" => {
                // List users
                json_response(object!({ users: get_all_users() }))
            },
            "POST" => {
                // Create user
                let data = req.body_json().await?;
                let new_user = create_user(data);
                json_response(new_user).status(StatusCode::CREATED)
            },
            "DELETE" => {
                // Bulk delete (if needed)
                json_response(object!({ deleted: true }))
            },
            _ => {
                // Method not allowed
                text_response("Method not allowed")
                    .status(StatusCode::METHOD_NOT_ALLOWED)
            }
        }
    }
}

// For specific resource operations
endpoint! {
    APP.url("/api/users/<id>"),
    
    /// Handles operations on a specific user
    pub user_handler <HTTP> {
        let user_id = req.param("id").unwrap_or_default();
        
        match req.method() {
            "GET" => get_user(user_id),
            "PUT" => update_user(user_id, req).await,
            "DELETE" => delete_user(user_id),
            _ => method_not_allowed()
        }
    }
}
```

#### Design Philosophy

This constraint reflects RESTful principles:
- **Resource-Oriented**: URLs represent resources, not actions
- **Simplicity**: One handler per resource is easier to maintain
- **Performance**: No overhead searching multiple handlers
- **Clarity**: All logic for a resource is in one place

Think of URLs as **resources** with methods as **actions** on those resources, not as method-URL combinations.

## 5. Middleware Patterns

### Basic Middleware (Struct Style)
```rust
middleware! {
    /// Logs all incoming requests
    pub LogRequest <Protocol> {
        println!("Request: {} {}", req.method(), req.path());
        
        // Continue to next middleware/endpoint
        next(req).await
    }
}
```

### Short-Circuit Middleware
```rust
middleware! {
    /// Authentication check - stops chain if unauthorized
    pub AuthCheck <HTTP> {
        if !is_authorized(&req) {
            // Don't call next() - return early
            req.response = text_response("Unauthorized");
            req
        } else {
            // Continue chain
            next(req).await
        }
    }
}
```

### Data-Passing Middleware
```rust
middleware! {
    /// Sets values for downstream middleware/endpoints
    pub SetUserContext <HTTP> {
        // Set values in locals
        req.locals.set("user_id", "user123".to_string());
        
        next(req).await
    }
}
```

### Global Middleware and the `..` Pattern

Hotaru supports global middleware defined at the application level that can be inherited by endpoints using the `..` pattern.

#### Defining Global Middleware
```rust
// Define middleware as usual
middleware! {
    pub GlobalLogger <HTTP> {
        println!("[LOG] {}", req.path());
        next(req).await
    }
}

middleware! {
    pub GlobalMetrics <HTTP> {
        let start = std::time::Instant::now();
        let result = next(req).await;
        println!("[METRICS] {:?}", start.elapsed());
        result
    }
}

// Register globally when creating the app
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3003")
        .single_protocol(
            ProtocolBuilder::new(HTTP::server(HttpSafety::default()))
                .append_middleware::<GlobalLogger>()
                .append_middleware::<GlobalMetrics>()
        )
        .build()
});
```

#### Automatic Inheritance and the `..` Pattern

Endpoints automatically inherit global middleware when no `middleware` field is specified. The `..` pattern provides control when combining global and local middleware:

```rust
// DEFAULT: No middleware field = automatic global inheritance
endpoint! {
    APP.url("/api/standard"),
    
    pub standard_endpoint <HTTP> {
        // Automatically inherits: GlobalLogger → GlobalMetrics → Handler
        text_response("Uses global middleware by default")
    }
}

// Pattern 1: Global first, then local
endpoint! {
    APP.url("/api/data"),
    middleware = [.., LocalAuth],
    
    pub data_endpoint <HTTP> {
        // Order: GlobalLogger → GlobalMetrics → LocalAuth → Handler
        json_response(object!({ data: "secure" }))
    }
}

// Pattern 2: Local first, then global
endpoint! {
    APP.url("/api/public"),
    middleware = [RateLimit, ..],
    
    pub public_endpoint <HTTP> {
        // Order: RateLimit → GlobalLogger → GlobalMetrics → Handler
        text_response("Public data")
    }
}

// Pattern 3: Sandwich pattern
endpoint! {
    APP.url("/api/complex"),
    middleware = [Auth, .., Cache],
    
    pub complex_endpoint <HTTP> {
        // Order: Auth → GlobalLogger → GlobalMetrics → Cache → Handler
        json_response(object!({ complex: true }))
    }
}

// Pattern 4: Explicit global only (same as default)
endpoint! {
    APP.url("/api/logged"),
    middleware = [..],
    
    pub logged_endpoint <HTTP> {
        // Same as having no middleware field
        // Order: GlobalLogger → GlobalMetrics → Handler
        text_response("Logged")
    }
}

// Pattern 5: OPT-OUT - Only local, no global
endpoint! {
    APP.url("/api/isolated"),
    middleware = [OnlyLocal],  // No .. means NO global inheritance
    
    pub isolated_endpoint <HTTP> {
        // IMPORTANT: Without .., global middleware is excluded
        // Order: OnlyLocal → Handler (no global middleware)
        text_response("Isolated")
    }
}
```

#### Best Practices for Global Middleware

1. **Use for Cross-Cutting Concerns**: Global middleware is ideal for:
   - Request/response logging
   - Metrics collection
   - CORS headers
   - Request ID generation
   - Rate limiting

2. **Understand the Default**: Remember that endpoints without middleware specification automatically inherit global middleware

3. **Explicit Opt-Out**: To exclude global middleware, specify local middleware WITHOUT `..`:
   ```rust
   middleware = [OnlyThis]  // No global inheritance
   ```

4. **Order Control**: Use `..` to precisely control execution order:
   - `[.., Local]` - Global concerns first (logging, metrics)
   - `[Local, ..]` - Local concerns first (auth, validation)  
   - `[Local1, .., Local2]` - Sandwich pattern for complex flows

5. **Performance**: Global middleware runs for most requests, so keep it lightweight

6. **Documentation**: Comment when opting out to explain why global middleware is excluded

## 6. Documentation Conventions

### Critical: Doc Comment Placement
**Doc comments MUST be placed INSIDE the macro, AFTER the URL line, and BEFORE the handler name.**

❌ **WRONG - Outside the macro:**
```rust
/// Gets a list of all users  // ❌ This won't work!
endpoint! {
    APP.url("/users"),
    
    pub get_users <HTTP> {
        // Implementation
    }
}
```

✅ **CORRECT - Inside macro, after URL, before handler:**
```rust
endpoint! {
    APP.url("/users"),
    
    /// Gets a list of all users  // ✅ Correct placement!
    pub get_users <HTTP> {
        // Implementation
    }
}
```

### Doc Comment Rules

1. **Placement Order**:
   - First: URL declaration (`APP.url("/path")`)
   - Second: Optional middleware declaration
   - Third: Doc comments (`///`)
   - Fourth: Handler declaration (`pub handler_name <PROTOCOL>`)

2. **Complete Example with All Elements**:
```rust
endpoint! {
    APP.url("/api/users/<id>"),
    middleware = [AuthCheck, RateLimit],
    
    /// Retrieves user information by ID
    /// 
    /// Returns a JSON object with user details
    pub get_user <HTTP> {
        // Implementation
    }
}
```

3. **Middleware Documentation**:
```rust
middleware! {
    /// Validates request headers and authentication
    pub ValidateHeaders <HTTP> {
        // Implementation
    }
}
```

### Structured API Documentation
```rust
endpoint! {
    APP.url("/api/users/<int:id>"),
    
    /// # Request
    /// 
    /// `GET /api/users/{id}`
    /// 
    /// # Response
    /// 
    /// `JSON { "id": 123, "name": "John" }`
    /// 
    /// # Comments
    /// 
    /// This endpoint retrieves user details by ID
    pub get_user <HTTP> {
        // Implementation
    }
}
```

**Note:** IDEs may show errors for doc comments inside macros, but they compile correctly.

## 7. Response Patterns

### Text Response
```rust
text_response("Plain text response")
```

### JSON Response
```rust
// Using object! macro (preferred)
json_response(object!({
    status: "success",
    data: {
        id: 123,
        name: "John"
    }
}))

// Using serde_json
use serde_json::json;
json_response(json!({
    "status": "success",
    "data": {}
}))
```

### Response with Status Code
```rust
// Use .status() method, not .with_status()
json_response(object!({
    status: "error",
    message: "Not found"
})).status(StatusCode::NOT_FOUND)
```

### Response with Headers and Cookies
```rust
text_response("Response")
    .add_header("X-Custom", "value")
    .add_cookie("session", Cookie::new("abc123".to_string())
        .path("/")
        .max_age(3600))
```

## 8. Request Handling

### Method Checking
```rust
// Direct comparison with method constants
if req.method() == POST {
    // Handle POST
} else if req.method() == GET {
    // Handle GET
}
```

### Form Data
```rust
match req.form().await {
    Some(form) => {
        let username = form.data.get("username")
            .map(|s| s.as_str())
            .unwrap_or("anonymous");
        
        text_response(format!("Hello, {}!", username))
    }
    None => {
        text_response("Invalid form data")
    }
}
```

### JSON Data
```rust
// Generic JSON
match req.json::<serde_json::Value>().await {
    Some(json_data) => {
        // Process JSON
    }
    None => {
        text_response("Invalid JSON")
    }
}
```

### Headers
```rust
// Note: HeaderValue uses .as_str(), not .to_str()
let auth = req.headers()
    .get("Authorization")
    .and_then(|h| h.as_str().ok())
    .unwrap_or("");
```

## 9. Template & Data Management - Akari Integration

### Akari Overview
Akari is Hotaru's integrated templating and JSON manipulation library. Use it for:
- HTML template rendering with `akari_render!`
- JSON creation with `object!` macro
- JSON manipulation with `Value` type
- Template components and reusability

### JSON Best Practices

#### Use object! for Clean JSON Creation
```rust
// ✅ Good: Clear, readable JSON structure
let response = object!({
    status: "success",
    data: {
        id: user_id,
        name: username,
        settings: user_settings
    }
});

// ❌ Avoid: Manual JSON construction
let response = format!(r#"{{"status":"success","data":{{"id":"{}"}}}}"#, user_id);
```

#### Dynamic JSON Manipulation
```rust
// Start with base structure
let mut data = object!({
    items: [],
    total: 0
});

// Add items dynamically
if let Some(Value::Array(items)) = data.get_mut("items") {
    items.push(object!({ id: 1, name: "Item 1" }));
}

// Update counters
data["total"] = Value::Number(items.len() as i64);
```

### Template Rendering Best Practices

#### Use akari_render! for HTML Responses
```rust
endpoint! {
    APP.url("/profile"),
    
    pub profile_page <HTTP> {
        // ✅ Good: Type-safe template rendering
        let html = akari_render!("templates/profile.html", {
            user: user_data,
            title: "User Profile"
        });
        
        http_response()
            .body(html)
            .header("content-type", "text/html; charset=utf-8")
    }
}
```

#### Component-Based Templates
```rust
// Define reusable components
const HEADER_COMPONENT: &str = r#"
<header>
    <h1>{{ title }}</h1>
    <nav>{{ nav_items }}</nav>
</header>
"#;

// Use in endpoints
let header = akari_render_string!(HEADER_COMPONENT, {
    title: "My App",
    nav_items: nav_html
});
```

### Common Patterns

#### JSON API Responses
```rust
// Consistent API response structure
fn api_success<T>(data: T) -> HttpResponse 
where T: Into<Value> 
{
    json_response(object!({
        status: "success",
        data: data.into(),
        timestamp: current_timestamp()
    }))
}

fn api_error(message: &str, code: &str) -> HttpResponse {
    json_response(object!({
        status: "error",
        error: {
            message: message,
            code: code
        }
    })).status(StatusCode::BAD_REQUEST)
}
```

#### Template Data Preparation
```rust
// Prepare complex data for templates
let template_data = object!({
    page: {
        title: page_title,
        meta_description: description
    },
    user: if authenticated {
        object!({ name: username, id: user_id })
    } else {
        Value::Null
    },
    navigation: build_nav_items()
});
```

### Performance Considerations

1. **Template Caching**: Templates are cached in release mode - no need for manual caching
2. **Lazy Loading**: Use `Value::Null` for optional fields instead of empty objects
3. **Streaming**: For large JSON responses, consider `akari::stream` for chunked output

### Security Best Practices

1. **Auto-Escaping**: Akari templates auto-escape HTML by default
2. **Raw Output**: Use `{{ value | raw }}` only when absolutely necessary
3. **JSON Validation**: Validate JSON input with schemas before processing

### Program Files
- Configuration files go in `programfiles/`
- Template files typically in `templates/` or `programfiles/templates/`
- Runtime data can also be stored in `programfiles/`
- Auto-copy mechanism handles synchronization

### Complete Example
```rust
use hotaru::prelude::*;
use akari::{object, Value, akari_render};

endpoint! {
    APP.url("/api/dashboard"),
    
    pub dashboard <HTTP> {
        let accept = req.header_str("accept").unwrap_or("application/json");
        
        // Prepare data
        let dashboard_data = object!({
            metrics: fetch_metrics(),
            recent_activity: get_recent_activity(),
            user: get_current_user(&req)
        });
        
        // Return JSON or HTML based on Accept header
        if accept.contains("text/html") {
            let html = akari_render!("templates/dashboard.html", {
                data: dashboard_data,
                page_title: "Dashboard"
            });
            
            http_response()
                .body(html)
                .header("content-type", "text/html; charset=utf-8")
        } else {
            json_response(dashboard_data)
        }
    }
}
```

### Learn More
For comprehensive Akari documentation including advanced features, visit: [fds.rs/akari](https://fds.rs/akari/tutorial/0.2.7)

## 10. Async & Performance

### Automatic Async
All endpoints and middleware are automatically async:
```rust
endpoint! {
    APP.url("/async"),
    
    pub async_endpoint <HTTP> {
        // This is automatically async
        use tokio::time::{sleep, Duration};
        
        sleep(Duration::from_millis(100)).await;
        text_response("Done")
    }
}
```

### Worker Thread Configuration
```rust
// Custom worker threads for I/O-heavy workloads
#[tokio::main(worker_threads = 16)]
async fn main() {
    APP.clone().run().await;
}
```

## 11. Security Patterns

### HTTP Safety Configuration
```rust
endpoint! {
    APP.url("/api/data"),
    config = [HttpSafety::new().with_allowed_method(GET)],
    
    pub get_only <HTTP> {
        text_response("GET only")
    }
}

// Multiple methods
config = [HttpSafety::new().with_allowed_methods(vec![GET, POST])]
```

### CORS
```rust
use htmstd::Cors;

endpoint! {
    APP.url("/api/public"),
    middleware = [Cors],
    
    pub public_api <HTTP> {
        json_response(object!({ status: "success" }))
    }
}
```

## 12. Common Pitfalls & Solutions

### Borrow/Move Errors
When using values in `object!` macro, use references:
```rust
// Wrong - causes move error
json_response(object!({
    id: id,
    name: format!("User {}", id)  // id moved here
}))

// Correct - use reference
json_response(object!({
    id: &id,
    name: format!("User {}", id)
}))
```

### Method Names
- Use `.status()` not `.with_status()`
- Use `.as_str()` not `.to_str()` for HeaderValue
- Compare methods directly: `req.method() == POST`

### Imports
Always import in this order:
1. `use hotaru::prelude::*;`
2. `use hotaru::http::*;`
3. External crates
4. Standard library

## 13. Testing Patterns

### Test Commands in main.rs
```rust
#[tokio::main]
async fn main() {
    println!("Test Commands:");
    println!("  curl http://localhost:3003/");
    println!("  curl -X POST http://localhost:3003/form -d 'username=test'");
    
    APP.clone().run().await;
}
```

## 14. Best Practices Summary

### Do's
✅ Use prelude imports  
✅ Use brace syntax `{}` for `endpoint!` and `middleware!` macros  
✅ Format endpoints as functions, middleware as structs  
✅ Use explicit type annotations  
✅ Store configuration in `programfiles/`  
✅ Use Akari for HTTP templates  
✅ Document inside macros  
✅ Trust automatic async conversion  
✅ Use response helper functions  
✅ Handle errors gracefully  
✅ Use port 3003 as the default convention  
✅ Remember endpoints receive contexts, not raw messages  

### Don'ts
❌ Use parenthesis syntax `()` for macros (always use braces `{}`)  
❌ Block async without `spawn_blocking`  
❌ Mix protocol logic unnecessarily  
❌ Use implicit types in public APIs  
❌ Put doc comments outside macros  
❌ Return `Result<Response>` from endpoints  
❌ Use `with_status()` (use `status()` instead)  
❌ Use `to_str()` on HeaderValue (use `as_str()`)  

## 15. Common Pitfalls and Solutions

### URL Uniqueness Constraint

**❌ WRONG - Multiple endpoints with same URL:**
```rust
// Only the last endpoint will work!
endpoint! { 
    APP.url("/api/users"), 
    pub list_users <HTTP> { /* GET logic */ }
}

endpoint! { 
    APP.url("/api/users"), 
    pub create_user <HTTP> { /* POST logic - overwrites list_users! */ }
}
```

**✅ CORRECT - Single handler with method routing:**
```rust
endpoint! {
    APP.url("/api/users"),
    
    pub users_handler <HTTP> {
        match req.method() {
            HttpMethod::GET => { /* list logic */ },
            HttpMethod::POST => { /* create logic */ },
            HttpMethod::PUT => { /* update logic */ },
            HttpMethod::DELETE => { /* delete logic */ },
            _ => { /* method not allowed */ }
        }
    }
}
```

### JSON Handling with Akari

**❌ WRONG - Using serde:**
```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    name: String,
    age: u32,
}

// Unnecessary complexity!
let user: User = serde_json::from_str(&req.body_string()).unwrap();
```

**✅ CORRECT - Using Akari directly:**
```rust
// No struct needed!
let json_data = match req.json().await {
    Some(data) => data,
    None => return json_response(object!({ error: "No JSON data" }))
};
let name = json_data.get("name").string();
let age = json_data.get("age").numerical() as u32;

// Create response with object! macro
json_response(object!({
    name: name,
    age: age
}))
```

### Method Comparison

**❌ WRONG - String comparison:**
```rust
if req.method().to_string() == "POST" { ... }
```

**✅ CORRECT - Direct enum comparison:**
```rust
if req.method() == HttpMethod::POST { ... }
// or
match req.method() {
    HttpMethod::GET => { ... },
    HttpMethod::POST => { ... },
    _ => { ... }
}
```

### Akari Value Access

**❌ WRONG - Expecting Option:**
```rust
// get() returns &Value, not Option<&Value>
if let Some(value) = json_data.get("field") { ... }  // Won't compile!
```

**✅ CORRECT - Direct access:**
```rust
let value = json_data.get("field");  // Returns &Value
if !value.is_none() {
    let string_val = value.string();
}
```

### Query Parameter Access

**❌ WRONG - Non-existent methods:**
```rust
let param = req.get_url_args("key");  // Doesn't exist
let param = req.request.meta.get_url_args("key");  // Wrong
```

**✅ CORRECT - Using query method:**
```rust
let param = req.query("key");  // Returns Option<String>
```

### Minimal Dependencies

**❌ WRONG - Unnecessary dependencies:**
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }  # Not needed!
serde_json = "1.0"  # Not needed!
```

**✅ CORRECT - Minimal dependencies:**
```toml
[dependencies]
hotaru = { path = "../hotaru" }
akari = "0.2"  # Provides JSON handling
tokio = { version = "1", features = ["full"] }
once_cell = "1"
ctor = "0.4"  # Required for endpoint registration
```

## 16. Complete Working Example

See `hotaru_style_guide/src/main_simple.rs` for a complete, compilable example demonstrating all these patterns.

## Conclusion

This style guide represents the idiomatic way to write Hotaru applications. Following these patterns ensures consistency, maintainability, and optimal performance. The macro system and automatic async conversion handle much of the boilerplate, allowing you to focus on your application logic.
