# Hotaru HTTP Documentation

This document covers HTTP/1.1 specific features in Hotaru. For protocol-agnostic concepts, see QUICK_TUTORIAL.md. For HTTP/2 and WebSocket, see the hotaru_http_advanced crate.

## Table of Contents
1. [HTTP Fundamentals](#1-http-fundamentals)
2. [JSON with Akari](#2-json-with-akari)
3. [Forms & File Handling](#3-forms--file-handling)
4. [HTTP Security](#4-http-security)
5. [Sessions & Cookies](#5-sessions--cookies)

---

## 1. HTTP Fundamentals

### HttpContext Structure

In Hotaru, every HTTP endpoint receives an `HttpContext` (aliased as `req` in endpoints):

```rust
endpoint! {
    APP.url("/example"),
    
    pub example <HTTP> {
        // 'req' is an HttpContext containing:
        // - request: HttpRequest (parsed HTTP request)
        // - response: HttpResponse (to be sent)
        // - app: Arc<App> (application instance)
        // - endpoint: Arc<Url> (matched endpoint)
        
        text_response("Hello")
    }
}
```

Optional fn-style (request naming):

```rust
endpoint! {
    APP.url("/example"),
    pub fn example(req: HTTP) {
        text_response("Hello")
    }
}
```

### Request Access

```rust
endpoint! {
    APP.url("/request-info"),
    
    pub request_info <HTTP> {
        // Method
        let method = req.method(); // Returns HttpMethod enum
        
        // Path
        let path = req.request.meta.path();
        
        // Headers
        let user_agent = req.request.meta.get_header("User-Agent")
            .unwrap_or_default();
        
        // Raw body access (if needed)
        req.parse_body().await; // Parse body first
        let body = &req.request.body;
        
        json_response(object!({
            method: method.to_string(),
            path: path,
            user_agent: user_agent
        }))
    }
}
```

### Response Building

```rust
use hotaru_core::http::response::HttpResponse;
use hotaru_core::http::body::HttpBody;
use hotaru_core::http::http_value::{HttpContentType, StatusCode};

endpoint! {
    APP.url("/custom-response"),
    
    pub custom_response <HTTP> {
        // Building a custom response
        let mut response = HttpResponse::default();
        
        // Set status
        response = response.status(StatusCode::CREATED);
        
        // Add headers
        response = response.add_header("X-Custom", "Value");
        
        // Set content type
        response = response.content_type(HttpContentType::ApplicationJson());
        
        // Set body
        response.body = HttpBody::Text(r#"{"message": "Created"}"#.to_string());
        
        response
    }
}
```

### Path Parameters

```rust
endpoint! {
    APP.url("/users/{id}/posts/{post_id}"),
    
    pub get_user_post <HTTP> {
        // Extract path parameters
        let user_id = req.request.meta.get_param("id")
            .unwrap_or_default();
        let post_id = req.request.meta.get_param("post_id")
            .unwrap_or_default();
        
        json_response(object!({
            user_id: user_id,
            post_id: post_id
        }))
    }
}
```

### Query Parameters

```rust
endpoint! {
    APP.url("/search"),
    
    pub search <HTTP> {
        // Get query parameters from the URL
        let query_params = req.request.meta.get_query_params();
        
        // Access specific parameter
        let search_term = query_params.get("q")
            .map(|v| v.as_str())
            .unwrap_or("");
        
        let page = query_params.get("page")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);
        
        json_response(object!({
            search: search_term,
            page: page
        }))
    }
}
```

---

## 2. JSON with Akari

Hotaru uses Akari (v0.2.6) for JSON handling without serde dependencies.

### The object! Macro

```rust
use akari::{Value, object};

endpoint! {
    APP.url("/json-example"),
    
    pub json_example <HTTP> {
        // Create JSON using object! macro
        let data = object!({
            name: "Hotaru",
            version: "0.7.0",
            features: ["fast", "multi-protocol", "middleware"],
            metadata: {
                author: "Redstone",
                year: 2024
            },
            active: true,
            count: 42
        });
        
        json_response(data)
    }
}
```

### Parsing JSON Requests

```rust
endpoint! {
    APP.url("/api/users"),
    
    pub create_user <HTTP> {
        if req.method() != HttpMethod::POST {
            return json_response(object!({
                error: "Method not allowed"
            }));
        }
        
        // Parse JSON from request body
        req.parse_body().await;
        
        let json_body = match &req.request.body {
            HttpBody::Json(value) => value,
            _ => return json_response(object!({
                error: "Invalid JSON"
            }))
        };
        
        // Access JSON fields
        let name = json_body.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let age = json_body.get("age")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        
        let email = json_body.get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Validate
        if name.is_empty() || email.is_empty() {
            return json_response(object!({
                error: "Name and email are required"
            }));
        }
        
        // Process...
        json_response(object!({
            message: "User created",
            user: {
                name: name,
                age: age,
                email: email
            }
        }))
    }
}
```

### Working with Akari Values

```rust
use akari::Value;

// Creating values
let string_val = Value::String("Hello".to_string());
let number_val = Value::Number(42.into());
let bool_val = Value::Bool(true);
let null_val = Value::Null;

// Arrays
let array = Value::Array(vec![
    Value::String("item1".to_string()),
    Value::String("item2".to_string()),
]);

// Objects
let mut map = std::collections::HashMap::new();
map.insert("key".to_string(), Value::String("value".to_string()));
let object = Value::Object(map);

// Type checking and conversion
if let Value::String(s) = &string_val {
    println!("String: {}", s);
}

// Safe access with as_* methods
let s = string_val.as_str(); // Returns Option<&str>
let n = number_val.as_i64(); // Returns Option<i64>
let b = bool_val.as_bool(); // Returns Option<bool>
```

### JSON Response Helpers

```rust
// Helper function for JSON responses
pub fn json_response(value: akari::Value) -> HttpResponse {
    let mut response = HttpResponse::default();
    response = response.content_type(HttpContentType::ApplicationJson());
    response.body = HttpBody::Json(value);
    response
}

// Usage
endpoint! {
    APP.url("/api/status"),
    
    pub api_status <HTTP> {
        json_response(object!({
            status: "ok",
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }))
    }
}
```

---

## 3. Forms & File Handling

### URL-Encoded Forms

```rust
endpoint! {
    APP.url("/form-submit"),
    
    pub handle_form <HTTP> {
        if req.method() != HttpMethod::POST {
            return text_response("Use POST to submit form");
        }
        
        req.parse_body().await;
        
        match &req.request.body {
            HttpBody::Form(form_data) => {
                // Access form fields
                let username = form_data.get("username")
                    .unwrap_or("");
                let password = form_data.get("password")
                    .unwrap_or("");
                
                json_response(object!({
                    message: "Form received",
                    username: username
                }))
            },
            _ => {
                json_response(object!({
                    error: "Expected form data"
                }))
            }
        }
    }
}
```

### Multipart File Upload

```rust
endpoint! {
    APP.url("/upload"),
    
    pub file_upload <HTTP> {
        if req.method() != HttpMethod::POST {
            return text_response("Use POST to upload");
        }
        
        req.parse_body().await;
        
        match &req.request.body {
            HttpBody::Files(multi_form) => {
                // Access uploaded files
                for (field_name, file_data) in multi_form.files.iter() {
                    println!("File field: {}", field_name);
                    println!("File size: {} bytes", file_data.len());
                    
                    // Save file
                    // std::fs::write(format!("uploads/{}", field_name), file_data)?;
                }
                
                // Access form fields
                for (key, value) in multi_form.fields.iter() {
                    println!("Field {}: {}", key, value);
                }
                
                json_response(object!({
                    message: "Files uploaded successfully"
                }))
            },
            _ => {
                json_response(object!({
                    error: "Expected multipart form data"
                }))
            }
        }
    }
}
```

### File Download

```rust
use std::fs;

endpoint! {
    APP.url("/download/{filename}"),
    
    pub file_download <HTTP> {
        let filename = req.request.meta.get_param("filename")
            .unwrap_or("file.txt");
        
        // Read file
        let file_path = format!("downloads/{}", filename);
        let file_data = match fs::read(&file_path) {
            Ok(data) => data,
            Err(_) => {
                let mut response = HttpResponse::default();
                response = response.status(StatusCode::NOT_FOUND);
                response.body = HttpBody::Text("File not found".to_string());
                return response;
            }
        };
        
        // Create download response
        let mut response = HttpResponse::default();
        response = response
            .content_type(HttpContentType::ApplicationOctetStream())
            .add_header("Content-Disposition", 
                format!("attachment; filename=\"{}\"", filename));
        response.body = HttpBody::Binary(file_data);
        response
    }
}
```

---

## 4. HTTP Security

### CORS Configuration

```rust
use htmstd::cors::cors::CorsMiddleware;

pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3000")
        .middleware(CorsMiddleware::new()
            .allow_origin("https://example.com")
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allow_headers(vec!["Content-Type", "Authorization"])
            .max_age(3600))
        .build()
});
```

### HttpSafety Settings

```rust
use hotaru_core::http::safety::HttpSafety;

pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3000")
        .config(HttpSafety::default()
            .max_body_size(10 * 1024 * 1024) // 10MB
            .allowed_methods(vec![
                HttpMethod::GET,
                HttpMethod::POST,
                HttpMethod::PUT,
                HttpMethod::DELETE,
            ])
            .allowed_content_types(vec![
                HttpContentType::ApplicationJson(),
                HttpContentType::ApplicationUrlEncodedForm(),
            ]))
        .build()
});
```

### Authentication Middleware

```rust
middleware! {
    pub auth_middleware {
        let auth_header = req.request.meta.get_header("Authorization")
            .unwrap_or_default();
        
        if !auth_header.starts_with("Bearer ") {
            let mut response = HttpResponse::default();
            response = response
                .status(StatusCode::UNAUTHORIZED)
                .add_header("WWW-Authenticate", "Bearer");
            response.body = HttpBody::Text("Unauthorized".to_string());
            return response;
        }
        
        let token = &auth_header[7..];
        
        // Validate token (implement your logic)
        if !validate_token(token) {
            let mut response = HttpResponse::default();
            response = response.status(StatusCode::FORBIDDEN);
            response.body = HttpBody::Text("Invalid token".to_string());
            return response;
        }
        
        // Continue to next middleware/handler
        next.run(req).await
    }
}

fn validate_token(token: &str) -> bool {
    // Implement token validation
    token == "valid-token-123"
}
```

### Rate Limiting

```rust
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::{Instant, Duration};

middleware! {
    pub rate_limit {
        static REQUESTS: Lazy<Mutex<HashMap<String, Vec<Instant>>>> = 
            Lazy::new(|| Mutex::new(HashMap::new()));
        
        // Get client IP
        let client_ip = req.request.meta.get_header("X-Forwarded-For")
            .or_else(|| req.request.meta.get_header("X-Real-IP"))
            .unwrap_or("unknown");
        
        let mut requests = REQUESTS.lock().unwrap();
        let now = Instant::now();
        let window = Duration::from_secs(60); // 1 minute window
        
        // Clean old requests
        let client_requests = requests.entry(client_ip.to_string())
            .or_insert_with(Vec::new);
        client_requests.retain(|&t| now.duration_since(t) < window);
        
        // Check rate limit (e.g., 60 requests per minute)
        if client_requests.len() >= 60 {
            let mut response = HttpResponse::default();
            response = response
                .status(StatusCode::TOO_MANY_REQUESTS)
                .add_header("Retry-After", "60");
            response.body = HttpBody::Text("Rate limit exceeded".to_string());
            return response;
        }
        
        // Record this request
        client_requests.push(now);
        drop(requests);
        
        next.run(req).await
    }
}
```

---

## 5. Sessions & Cookies

### Cookie Management

```rust
use hotaru_core::http::cookie::Cookie;

endpoint! {
    APP.url("/set-cookie"),
    
    pub set_cookie <HTTP> {
        let mut response = HttpResponse::default();
        
        // Create a cookie
        let session_cookie = Cookie::new("session_id", "abc123")
            .path("/")
            .max_age(3600) // 1 hour
            .http_only(true)
            .secure(true)
            .same_site("Strict");
        
        // Add cookie to response
        response = response.add_cookie("session_id", session_cookie);
        response.body = HttpBody::Text("Cookie set".to_string());
        response
    }
}

endpoint! {
    APP.url("/get-cookie"),
    
    pub get_cookie <HTTP> {
        // Read cookies from request
        let cookies = req.request.meta.get_cookies();
        
        let session_id = cookies.get("session_id")
            .map(|c| c.value())
            .unwrap_or("not found");
        
        json_response(object!({
            session_id: session_id
        }))
    }
}
```

### Session Middleware

```rust
use htmstd::session::session::SessionMiddleware;
use htmstd::session::cookie_session::CookieSessionStore;

pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3000")
        .middleware(SessionMiddleware::new(
            CookieSessionStore::new("secret-key-change-this")
                .cookie_name("hotaru_session")
                .max_age(3600 * 24) // 24 hours
        ))
        .build()
});

endpoint! {
    APP.url("/login"),
    
    pub login <HTTP> {
        // Set session data
        req.set_session("user_id", "123");
        req.set_session("username", "alice");
        
        json_response(object!({
            message: "Logged in"
        }))
    }
}

endpoint! {
    APP.url("/profile"),
    
    pub profile <HTTP> {
        // Get session data
        let user_id = req.get_session("user_id")
            .unwrap_or("guest");
        let username = req.get_session("username")
            .unwrap_or("Guest User");
        
        json_response(object!({
            user_id: user_id,
            username: username
        }))
    }
}

endpoint! {
    APP.url("/logout"),
    
    pub logout <HTTP> {
        // Clear session
        req.clear_session();
        
        json_response(object!({
            message: "Logged out"
        }))
    }
}
```

---

## Common Patterns

### Error Handling

```rust
endpoint! {
    APP.url("/api/resource/{id}"),
    
    pub get_resource <HTTP> {
        let id = req.request.meta.get_param("id")
            .and_then(|s| s.parse::<u32>().ok());
        
        let id = match id {
            Some(id) => id,
            None => {
                let mut response = HttpResponse::default();
                response = response.status(StatusCode::BAD_REQUEST);
                response.body = HttpBody::Json(object!({
                    error: "Invalid ID format"
                }));
                return response;
            }
        };
        
        // Fetch resource...
        match fetch_resource(id) {
            Ok(resource) => json_response(resource),
            Err(e) => {
                let mut response = HttpResponse::default();
                response = response.status(StatusCode::INTERNAL_SERVER_ERROR);
                response.body = HttpBody::Json(object!({
                    error: e.to_string()
                }));
                response
            }
        }
    }
}
```

### Content Negotiation

```rust
endpoint! {
    APP.url("/api/data"),
    
    pub get_data <HTTP> {
        let accept = req.request.meta.get_header("Accept")
            .unwrap_or("application/json");
        
        let data = object!({
            message: "Hello",
            timestamp: 1234567890
        });
        
        if accept.contains("application/json") {
            json_response(data)
        } else if accept.contains("text/plain") {
            text_response(format!("Message: {}", data.get("message").unwrap()))
        } else if accept.contains("text/html") {
            html_response(format!(
                "<html><body><h1>{}</h1></body></html>",
                data.get("message").unwrap()
            ))
        } else {
            let mut response = HttpResponse::default();
            response = response.status(StatusCode::NOT_ACCEPTABLE);
            response.body = HttpBody::Text("Unsupported media type".to_string());
            response
        }
    }
}
```

---

## Testing HTTP Endpoints

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use hotaru_core::http::request::HttpRequest;
    
    #[tokio::test]
    async fn test_endpoint() {
        // Create a mock request
        let request = HttpRequest::builder()
            .method(HttpMethod::GET)
            .path("/api/users")
            .header("Authorization", "Bearer test-token")
            .build();
        
        // Create context
        let context = HttpContext::new_test(request);
        
        // Run endpoint
        let response = list_users.run(context).await;
        
        // Assert response
        assert_eq!(response.status(), StatusCode::OK);
        
        if let HttpBody::Json(json) = &response.body {
            assert!(json.get("users").is_some());
        } else {
            panic!("Expected JSON response");
        }
    }
}
```

---

## Performance Tips

1. **Parse body only when needed** - Call `req.parse_body().await` only for endpoints that need the body
2. **Use streaming for large files** - Don't load entire files into memory
3. **Cache static responses** - Use lazy_static for responses that don't change
4. **Validate early** - Check method, content-type, and size limits before processing
5. **Use connection pooling** - For database and external API calls

---

## Migration from Other Frameworks

### From Actix-Web

```rust
// Actix-Web
#[get("/users/{id}")]
async fn get_user(id: web::Path<u32>) -> impl Responder {
    HttpResponse::Ok().json(User { id: *id })
}

// Hotaru
endpoint! {
    APP.url("/users/{id}"),
    
    pub get_user <HTTP> {
        let id = req.request.meta.get_param("id")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        
        json_response(object!({ id: id }))
    }
}
```

### From Rocket

```rust
// Rocket
#[get("/hello/<name>")]
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Hotaru
endpoint! {
    APP.url("/hello/{name}"),
    
    pub hello <HTTP> {
        let name = req.request.meta.get_param("name")
            .unwrap_or("World");
        text_response(format!("Hello, {}!", name))
    }
}
```

---

## Next Steps

- For HTTP/2 and WebSocket support, see the `hotaru_http_advanced` crate
- For protocol-agnostic concepts, see QUICK_TUTORIAL.md
- For framework internals, see HOTARU_STYLE.md
