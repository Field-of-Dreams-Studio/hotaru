# Hotaru Quick Tutorial

A focused 4-part tutorial to get you productive with Hotaru. For detailed HTTP features, see HOTARU_HTTP_DOC.md. For HTTP/2 and WebSocket, see the hotaru_http_advanced crate.

## Table of Contents
1. [Part 1: Getting Started](#part-1-getting-started)
2. [Part 2: Advanced Routing & State Management](#part-2-advanced-routing--state-management)
3. [Part 3: Middleware System](#part-3-middleware-system)
4. [Part 4: Protocol Abstraction](#part-4-protocol-abstraction)

---

## Part 1: Getting Started

### Your First Hotaru Server

```rust
use hotaru::prelude::*;
use hotaru::http::*;

// Define your application
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3000")
        .build()
});

// Your first endpoint
endpoint! {
    APP.url("/"),
    
    pub hello_world <HTTP> {
        text_response("Hello, Hotaru!")
    }
}

#[tokio::main]
async fn main() {
    println!("Server running at http://127.0.0.1:3000");
    APP.clone().run().await;
}
```

Optional fn-style (request naming):

```rust
endpoint! {
    APP.url("/"),
    pub fn hello_world(req: HTTP) {
        text_response("Hello, Hotaru!")
    }
}
```

### Basic Routing

```rust
// Path parameters
endpoint! {
    APP.url("/users/{id}"),
    
    pub get_user <HTTP> {
        let id = req.request.meta.get_param("id")
            .unwrap_or("unknown");
        text_response(format!("User ID: {}", id))
    }
}

// Multiple parameters
endpoint! {
    APP.url("/posts/{year}/{month}/{slug}"),
    
    pub get_post <HTTP> {
        let year = req.request.meta.get_param("year").unwrap_or("2024");
        let month = req.request.meta.get_param("month").unwrap_or("01");
        let slug = req.request.meta.get_param("slug").unwrap_or("post");
        
        text_response(format!("Post: {}/{}/{}", year, month, slug))
    }
}
```

### Query Parameters

```rust
endpoint! {
    APP.url("/search"),
    
    pub search <HTTP> {
        let params = req.request.meta.get_query_params();
        
        let query = params.get("q")
            .map(|s| s.as_str())
            .unwrap_or("");
            
        let limit = params.get("limit")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(10);
        
        json_response(object!({
            query: query,
            limit: limit
        }))
    }
}
```

### HTTP Methods

```rust
endpoint! {
    APP.url("/api/items"),
    
    pub items_handler <HTTP> {
        match req.method() {
            HttpMethod::GET => {
                json_response(object!({
                    message: "List items"
                }))
            },
            HttpMethod::POST => {
                json_response(object!({
                    message: "Create item"
                }))
            },
            HttpMethod::DELETE => {
                json_response(object!({
                    message: "Delete item"
                }))
            },
            _ => {
                let mut response = HttpResponse::default();
                response = response.status(StatusCode::METHOD_NOT_ALLOWED);
                response.body = HttpBody::Text("Method not allowed".to_string());
                response
            }
        }
    }
}
```

### Response Types

```rust
// Text response
endpoint! {
    APP.url("/text"),
    pub text_endpoint <HTTP> {
        text_response("Plain text response")
    }
}

// HTML response  
endpoint! {
    APP.url("/html"),
    pub html_endpoint <HTTP> {
        html_response("<h1>HTML Response</h1>")
    }
}

// JSON response (using Akari)
endpoint! {
    APP.url("/json"),
    pub json_endpoint <HTTP> {
        json_response(object!({
            status: "success",
            data: {
                id: 1,
                name: "Hotaru"
            }
        }))
    }
}
```

---

## Part 2: Advanced Routing & State Management

### CRUD Operations

```rust
use std::sync::RwLock;
use std::collections::HashMap;

// Shared state
static ITEMS: Lazy<RwLock<HashMap<u32, Item>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

static NEXT_ID: Lazy<RwLock<u32>> = Lazy::new(|| RwLock::new(1));

#[derive(Clone)]
struct Item {
    id: u32,
    name: String,
    description: String,
}

// Create
endpoint! {
    APP.url("/api/items"),
    
    pub create_item <HTTP> {
        if req.method() != HttpMethod::POST {
            return json_response(object!({ error: "Use POST" }));
        }
        
        req.parse_body().await;
        let json_body = match &req.request.body {
            HttpBody::Json(v) => v,
            _ => return json_response(object!({ error: "Invalid JSON" }))
        };
        
        let name = json_body.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let description = json_body.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        if name.is_empty() {
            return json_response(object!({ error: "Name required" }));
        }
        
        let mut id_lock = NEXT_ID.write().unwrap();
        let id = *id_lock;
        *id_lock += 1;
        drop(id_lock);
        
        let item = Item {
            id,
            name: name.to_string(),
            description: description.to_string(),
        };
        
        ITEMS.write().unwrap().insert(id, item.clone());
        
        json_response(object!({
            id: id,
            name: item.name,
            description: item.description
        }))
    }
}

// Read
endpoint! {
    APP.url("/api/items/{id}"),
    
    pub get_item <HTTP> {
        let id = req.request.meta.get_param("id")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        
        let items = ITEMS.read().unwrap();
        match items.get(&id) {
            Some(item) => json_response(object!({
                id: item.id,
                name: item.name.clone(),
                description: item.description.clone()
            })),
            None => {
                let mut response = HttpResponse::default();
                response = response.status(StatusCode::NOT_FOUND);
                response.body = HttpBody::Json(object!({ error: "Not found" }));
                response
            }
        }
    }
}

// Update
endpoint! {
    APP.url("/api/items/{id}"),
    
    pub update_item <HTTP> {
        if req.method() != HttpMethod::PUT {
            return json_response(object!({ error: "Use PUT" }));
        }
        
        let id = req.request.meta.get_param("id")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        
        req.parse_body().await;
        let json_body = match &req.request.body {
            HttpBody::Json(v) => v,
            _ => return json_response(object!({ error: "Invalid JSON" }))
        };
        
        let mut items = ITEMS.write().unwrap();
        match items.get_mut(&id) {
            Some(item) => {
                if let Some(name) = json_body.get("name").and_then(|v| v.as_str()) {
                    item.name = name.to_string();
                }
                if let Some(desc) = json_body.get("description").and_then(|v| v.as_str()) {
                    item.description = desc.to_string();
                }
                json_response(object!({
                    id: item.id,
                    name: item.name.clone(),
                    description: item.description.clone()
                }))
            },
            None => {
                let mut response = HttpResponse::default();
                response = response.status(StatusCode::NOT_FOUND);
                response.body = HttpBody::Json(object!({ error: "Not found" }));
                response
            }
        }
    }
}

// Delete
endpoint! {
    APP.url("/api/items/{id}"),
    
    pub delete_item <HTTP> {
        if req.method() != HttpMethod::DELETE {
            return json_response(object!({ error: "Use DELETE" }));
        }
        
        let id = req.request.meta.get_param("id")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        
        match ITEMS.write().unwrap().remove(&id) {
            Some(_) => json_response(object!({ message: "Deleted" })),
            None => {
                let mut response = HttpResponse::default();
                response = response.status(StatusCode::NOT_FOUND);
                response.body = HttpBody::Json(object!({ error: "Not found" }));
                response
            }
        }
    }
}

// List all
endpoint! {
    APP.url("/api/items"),
    
    pub list_items <HTTP> {
        if req.method() != HttpMethod::GET {
            return json_response(object!({ error: "Use GET" }));
        }
        
        let items = ITEMS.read().unwrap();
        let items_array: Vec<Value> = items.values().map(|item| {
            object!({
                id: item.id,
                name: item.name.clone(),
                description: item.description.clone()
            })
        }).collect();
        
        json_response(object!({
            items: items_array,
            count: items.len()
        }))
    }
}
```

### Using Akari Extensions for Type-Safe State

Akari (v0.2.6) provides an extensions system for type-safe shared state:

```rust
use akari::extensions::Extensions;

// Define your state types
#[derive(Clone)]
struct AppConfig {
    api_key: String,
    max_connections: usize,
}

#[derive(Clone)]
struct Database {
    // Your database connection
}

// Initialize extensions in your app
pub static APP: SApp = Lazy::new(|| {
    let mut extensions = Extensions::new();
    
    // Insert typed state
    extensions.insert(AppConfig {
        api_key: "secret-key".to_string(),
        max_connections: 100,
    });
    
    extensions.insert(Database {
        // Initialize database
    });
    
    App::new()
        .binding("127.0.0.1:3000")
        .extensions(extensions)
        .build()
});

// Access in endpoints
endpoint! {
    APP.url("/config"),
    
    pub get_config <HTTP> {
        // Get typed state from app
        let app = req.app().unwrap();
        let config = app.extensions.get::<AppConfig>().unwrap();
        
        json_response(object!({
            max_connections: config.max_connections
        }))
    }
}
```

### Error Handling Patterns

```rust
// Result type for operations
type ApiResult<T> = Result<T, ApiError>;

enum ApiError {
    NotFound,
    InvalidInput(String),
    Internal(String),
}

impl ApiError {
    fn to_response(&self) -> HttpResponse {
        let (status, message) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            ApiError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
        };
        
        let mut response = HttpResponse::default();
        response = response.status(status);
        response.body = HttpBody::Json(object!({ error: message }));
        response
    }
}

// Use in endpoints
endpoint! {
    APP.url("/api/validate/{id}"),
    
    pub validate_endpoint <HTTP> {
        let result: ApiResult<Value> = (|| {
            let id = req.request.meta.get_param("id")
                .ok_or(ApiError::InvalidInput("Missing ID".to_string()))?
                .parse::<u32>()
                .map_err(|_| ApiError::InvalidInput("Invalid ID format".to_string()))?;
            
            if id == 0 {
                return Err(ApiError::InvalidInput("ID cannot be zero".to_string()));
            }
            
            // Fetch from database...
            if id > 1000 {
                return Err(ApiError::NotFound);
            }
            
            Ok(object!({ id: id, valid: true }))
        })();
        
        match result {
            Ok(data) => json_response(data),
            Err(e) => e.to_response(),
        }
    }
}
```

---

## Part 3: Middleware System

### Understanding Middleware

Middleware in Hotaru are functions that intercept requests before they reach endpoints. They can:
- Modify requests
- Short-circuit with early responses
- Add headers to responses
- Track metrics
- Handle cross-cutting concerns

Syntax:

```rust
middleware! {
    pub Logger <HTTP> {
        next(req).await
    }
}

middleware! {
    pub fn Logger(req: HTTP) {
        next(req).await
    }
}
```

### Global Middleware

```rust
// Define middleware
middleware! {
    /// Logs all requests
    pub request_logger {
        println!("[{}] {} {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            req.method(),
            req.request.meta.path()
        );
        
        // Continue to next middleware/handler
        next.run(req).await
    }
}

middleware! {
    /// Adds response headers
    pub security_headers {
        let mut response = next.run(req).await;
        
        // Add security headers to all responses
        response = response
            .add_header("X-Frame-Options", "DENY")
            .add_header("X-Content-Type-Options", "nosniff")
            .add_header("X-XSS-Protection", "1; mode=block");
        
        response
    }
}

// Apply globally
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3000")
        .middleware(request_logger)
        .middleware(security_headers)
        .build()
});
```

### The `..` Pattern - Middleware Inheritance

The `..` pattern is Hotaru's elegant solution for middleware composition:

```rust
// Global middleware
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3000")
        .middleware(request_logger)
        .middleware(cors_middleware)
        .build()
});

// Endpoint automatically inherits global middleware
endpoint! {
    APP.url("/public"),
    // No middleware field = automatic inheritance
    
    pub public_endpoint <HTTP> {
        text_response("Uses global middleware")
    }
}

// Add middleware while keeping global ones
endpoint! {
    APP.url("/protected"),
    middleware = [.., auth_check],  // .. means "inherit global middleware"
    
    pub protected_endpoint <HTTP> {
        text_response("Uses global + auth middleware")
    }
}

// Replace all middleware
endpoint! {
    APP.url("/custom"),
    middleware = [timing_only],  // No .. = no inheritance
    
    pub custom_endpoint <HTTP> {
        text_response("Only uses timing middleware")
    }
}

// No middleware at all
endpoint! {
    APP.url("/raw"),
    middleware = [],  // Empty array = no middleware
    
    pub raw_endpoint <HTTP> {
        text_response("No middleware")
    }
}
```

### Common Middleware Patterns

#### Authentication
```rust
middleware! {
    pub auth_check {
        let token = req.request.meta.get_header("Authorization")
            .unwrap_or("");
        
        if !token.starts_with("Bearer ") {
            let mut response = HttpResponse::default();
            response = response
                .status(StatusCode::UNAUTHORIZED)
                .add_header("WWW-Authenticate", "Bearer");
            response.body = HttpBody::Text("Unauthorized".to_string());
            return response;
        }
        
        let token_value = &token[7..];
        // Validate token...
        
        next.run(req).await
    }
}
```

#### Request Timing
```rust
middleware! {
    pub timing {
        let start = std::time::Instant::now();
        
        let mut response = next.run(req).await;
        
        let duration = start.elapsed();
        response = response.add_header(
            "X-Response-Time", 
            format!("{}ms", duration.as_millis())
        );
        
        response
    }
}
```

#### Rate Limiting
```rust
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::Instant;

middleware! {
    pub rate_limit {
        static LAST_REQUEST: Lazy<Mutex<HashMap<String, Instant>>> = 
            Lazy::new(|| Mutex::new(HashMap::new()));
        
        let client = req.request.meta.get_header("X-Real-IP")
            .unwrap_or("unknown");
        
        let mut last_map = LAST_REQUEST.lock().unwrap();
        let now = Instant::now();
        
        if let Some(last) = last_map.get(client) {
            if now.duration_since(*last).as_secs() < 1 {
                let mut response = HttpResponse::default();
                response = response
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .add_header("Retry-After", "1");
                response.body = HttpBody::Text("Too many requests".to_string());
                return response;
            }
        }
        
        last_map.insert(client.to_string(), now);
        drop(last_map);
        
        next.run(req).await
    }
}
```

#### CORS
```rust
use htmstd::cors::cors::CorsMiddleware;

pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3000")
        .middleware(CorsMiddleware::default()
            .allow_origin("*")
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allow_headers(vec!["Content-Type", "Authorization"]))
        .build()
});
```

### Middleware Composition

```rust
// Combine multiple middleware for specific endpoints
middleware! {
    pub admin_only {
        // Check if user is admin
        let is_admin = check_admin_status(&req);
        if !is_admin {
            let mut response = HttpResponse::default();
            response = response.status(StatusCode::FORBIDDEN);
            response.body = HttpBody::Text("Admin access required".to_string());
            return response;
        }
        next.run(req).await
    }
}

endpoint! {
    APP.url("/admin/dashboard"),
    middleware = [.., auth_check, admin_only, rate_limit],
    
    pub admin_dashboard <HTTP> {
        json_response(object!({
            message: "Admin dashboard"
        }))
    }
}
```

---

## Part 4: Protocol Abstraction

### The Power of Protocol Abstraction

Hotaru's killer feature: running multiple protocols on the same port. This isn't just about HTTP/1.1 vs HTTP/2 - you can run completely different protocols (HTTP, WebSocket, TCP, gRPC) on the same port!

### Understanding Protocols

A protocol in Hotaru implements the `Protocol` trait:

```rust
use hotaru::prelude::*;

#[async_trait]
pub trait Protocol: Clone + Send + Sync + 'static {
    type Transport: Transport;
    type Stream: Stream;
    type Message: Message;
    type Context: RequestContext;
    
    // Detect if incoming bytes match this protocol
    fn detect(initial_bytes: &[u8]) -> bool;
    
    // Handle the connection
    async fn handle(
        &mut self,
        reader: BufReader<ReadHalf<TcpConnectionStream>>,
        writer: BufWriter<WriteHalf<TcpConnectionStream>>,
        app: Arc<App>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    fn role(&self) -> ProtocolRole;
}
```

### Multi-Protocol Application

```rust
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:8080")
        .handle(
            HandlerBuilder::new()
                // Add HTTP protocol
                .protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
                // Add custom TCP protocol
                .protocol(ProtocolBuilder::new(CustomTcp::new()))
                // Add WebSocket via H2PER
                .protocol(ProtocolBuilder::new(WebSocketProtocol::new()))
        )
        .build()
});
```

### Implementing a Custom Protocol

Let's implement a simple line-based chat protocol:

```rust
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Define the protocol
#[derive(Clone)]
pub struct SimpleChatProtocol {
    role: ProtocolRole,
}

impl SimpleChatProtocol {
    pub fn new() -> Self {
        Self { role: ProtocolRole::Server }
    }
}

// Message type
#[derive(Clone, Debug)]
pub enum ChatMessage {
    Join(String),
    Message(String),
    Leave,
}

impl ChatMessage {
    fn parse(data: &[u8]) -> Self {
        let text = String::from_utf8_lossy(data).trim().to_string();
        
        if text.starts_with("JOIN ") {
            ChatMessage::Join(text[5..].to_string())
        } else if text.starts_with("MSG ") {
            ChatMessage::Message(text[4..].to_string())
        } else {
            ChatMessage::Leave
        }
    }
}

// Implement Message trait
impl Message for ChatMessage {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), Box<dyn Error + Send + Sync>> {
        match self {
            ChatMessage::Join(name) => buf.extend_from_slice(format!("JOIN {}\n", name).as_bytes()),
            ChatMessage::Message(msg) => buf.extend_from_slice(format!("MSG {}\n", msg).as_bytes()),
            ChatMessage::Leave => buf.extend_from_slice(b"LEAVE\n"),
        }
        Ok(())
    }
    
    fn decode(buf: &mut bytes::BytesMut) -> Result<Option<Self>, Box<dyn Error + Send + Sync>> {
        if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let line = buf.split_to(pos + 1);
            Ok(Some(ChatMessage::parse(&line)))
        } else {
            Ok(None)
        }
    }
}

// Simple transport and stream (required by trait)
#[derive(Clone)]
pub struct ChatTransport;

impl Transport for ChatTransport {
    fn id(&self) -> i128 { 1 }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

#[derive(Clone)]
pub struct ChatStream;

impl Stream for ChatStream {
    fn id(&self) -> u32 { 0 }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// Context for request handling
pub struct ChatContext {
    role: ProtocolRole,
}

impl RequestContext for ChatContext {
    type Request = ChatMessage;
    type Response = ChatMessage;
    
    fn handle_error(&mut self) {}
    fn role(&self) -> ProtocolRole { self.role }
}

// Implement the Protocol trait
#[async_trait]
impl Protocol for SimpleChatProtocol {
    type Transport = ChatTransport;
    type Stream = ChatStream;
    type Message = ChatMessage;
    type Context = ChatContext;
    
    fn detect(initial_bytes: &[u8]) -> bool {
        let text = String::from_utf8_lossy(initial_bytes);
        text.starts_with("JOIN ") || 
        text.starts_with("MSG ") ||
        text.starts_with("CHAT:")  // Protocol identifier
    }
    
    fn role(&self) -> ProtocolRole {
        self.role
    }
    
    async fn handle(
        &mut self,
        reader: tokio::io::BufReader<tokio::io::ReadHalf<TcpConnectionStream>>,
        writer: tokio::io::BufWriter<tokio::io::WriteHalf<TcpConnectionStream>>,
        _app: Arc<App>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let stream = TcpConnectionStream::from_parts(reader.into_inner(), writer.into_inner());
        let (read_half, write_half) = stream.split();
        let mut reader = tokio::io::BufReader::new(read_half);
        let mut writer = tokio::io::BufWriter::new(write_half);
        
        // Send welcome
        writer.write_all(b"Welcome to chat! Commands: JOIN <name>, MSG <text>, LEAVE\n").await?;
        writer.flush().await?;
        
        let mut buffer = [0u8; 1024];
        loop {
            let n = match reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            
            let msg = ChatMessage::parse(&buffer[..n]);
            
            let response = match msg {
                ChatMessage::Join(name) => format!("Welcome, {}!\n", name),
                ChatMessage::Message(text) => format!("You said: {}\n", text),
                ChatMessage::Leave => {
                    writer.write_all(b"Goodbye!\n").await?;
                    writer.flush().await?;
                    break;
                }
            };
            
            writer.write_all(response.as_bytes()).await?;
            writer.flush().await?;
        }
        
        Ok(())
    }
}
```

### Shared State Between Protocols

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

// Shared state accessible by all protocols
#[derive(Clone)]
pub struct SharedState {
    pub connections: Arc<RwLock<Vec<String>>>,
    pub messages: Arc<RwLock<Vec<String>>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(Vec::new())),
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

// Use in HTTP endpoint
endpoint! {
    APP.url("/api/connections"),
    
    pub list_connections <HTTP> {
        let app = req.app().unwrap();
        let state = app.extensions.get::<SharedState>().unwrap();
        let connections = state.connections.read().await;
        
        json_response(object!({
            connections: connections.clone()
        }))
    }
}

// Use in custom protocol
impl SimpleChatProtocol {
    pub fn with_state(state: SharedState) -> Self {
        // Store and use shared state
        Self {
            role: ProtocolRole::Server,
            state,
        }
    }
}
```

### Real-World Example: Multi-Protocol Chat Server

See the complete implementation in `tutorial_examples/examples/ch4_multi_protocol.rs`:

```rust
// Single port serving both HTTP and TCP chat
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3003")
        .handle(
            HandlerBuilder::new()
                .protocol(ProtocolBuilder::new(TcpChat::with_room(
                    ProtocolRole::Server,
                    CHAT_ROOM.clone()
                )))
                .protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
        )
        .build()
});

// TCP clients connect with: nc localhost 3003
// HTTP clients browse to: http://localhost:3003
// Both share the same chat room!
```

### Protocol Detection

Hotaru examines the first few bytes of incoming connections to determine the protocol:

```rust
fn detect(initial_bytes: &[u8]) -> bool {
    // HTTP starts with method names
    if initial_bytes.starts_with(b"GET ") ||
       initial_bytes.starts_with(b"POST ") ||
       initial_bytes.starts_with(b"PUT ") {
        return false; // Let HTTP handler take it
    }
    
    // Our protocol starts with specific commands
    initial_bytes.starts_with(b"CHAT:") ||
    initial_bytes.starts_with(b"JOIN ")
}
```

### Benefits of Protocol Abstraction

1. **Single Port**: No need for multiple ports for different services
2. **Shared Resources**: Protocols can share state, connection pools, etc.
3. **Gradual Migration**: Add new protocols without changing existing ones
4. **Protocol Upgrade**: Start with HTTP, upgrade to WebSocket
5. **Flexibility**: Mix REST APIs, WebSocket, gRPC, and custom protocols

---

## Summary

You've learned Hotaru's four core concepts:

1. **Getting Started**: Basic routing, parameters, and responses
2. **Advanced Routing & State**: CRUD operations, Akari extensions, error handling
3. **Middleware System**: Global middleware, the `..` pattern, composition
4. **Protocol Abstraction**: Multi-protocol servers, custom protocols, shared state

### Next Steps

- **For HTTP details**: See HOTARU_HTTP_DOC.md
- **For HTTP/2 & WebSocket**: See hotaru_http_advanced crate
- **For examples**: Run the tutorial_examples crate
- **For production**: See HOTARU_STYLE.md for best practices

### Quick Reference

```rust
// Minimal Hotaru app
use hotaru::prelude::*;
use hotaru::http::*;

pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3000")
        .middleware(logger)           // Global middleware
        .build()
});

endpoint! {
    APP.url("/api/{resource}"),
    middleware = [.., auth],          // Inherit + add middleware
    
    pub api_endpoint <HTTP> {
        json_response(object!({
            resource: req.request.meta.get_param("resource")
        }))
    }
}

#[tokio::main]
async fn main() {
    APP.clone().run().await;
}
```

Welcome to Hotaru! 🚀
