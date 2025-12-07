// ============================================================================
// Hotaru Style Guide - Complete Example
// ============================================================================

// Import Conventions - Always use prelude
use hotaru::prelude::*;
use hotaru::http::*;
use htmstd::{Cors, CookieSession};
use tokio::time::sleep;
use std::time::Duration;

// ============================================================================
// Application Setup - Static APP Pattern
// ============================================================================

// Single protocol app
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3000")
        .build()
});

// Multi-protocol app example (commented out to avoid conflicts)
// pub static MULTI_APP: SApp = Lazy::new(|| {
//     App::new()
//         .binding("127.0.0.1:3001")
//         .handle(
//             HandlerBuilder::new()
//                 .protocol(ProtocolBuilder::new(HTTP::server(HttpSafety::default())))
//                 .protocol(ProtocolBuilder::new(h2per::prelude::HYPER1::new(ProtocolRole::Server)))
//                 .protocol(ProtocolBuilder::new(h2per::prelude::HYPER2::new(ProtocolRole::Server)))
//         )
//         .build()
// });

// ============================================================================
// Module imports
// ============================================================================
mod endpoints;
mod middleware;
mod response_patterns;
mod request_handling;
mod security;

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main(worker_threads = 16)]  // Custom worker threads for I/O-heavy workloads
async fn main() {
    println!("=================================================");
    println!("     Hotaru Style Guide Example Server");
    println!("=================================================");
    println!("Server running on: http://127.0.0.1:3000");
    println!("\nAvailable endpoints:");
    println!("  GET  /                    - Home page");
    println!("  GET  /docs                - API documentation");
    println!("  GET  /user/<id>           - Get user by ID");
    println!("  POST /api/json            - JSON echo endpoint");
    println!("  GET  /api/data            - Protected API endpoint");
    println!("  GET  /session             - Session example");
    println!("  GET  /async               - Async operation example");
    println!("\nTest commands:");
    println!("  curl http://localhost:3000/");
    println!("  curl http://localhost:3000/user/123");
    println!("  curl -X POST http://localhost:3000/api/json -H 'Content-Type: application/json' -d '{{\"test\":\"data\"}}'");
    println!("=================================================\n");
    
    APP.clone().run().await;
}

// ============================================================================
// Basic Endpoints - Function Style
// ============================================================================

endpoint! {
    APP.url("/"),
    
    /// Home page endpoint
    pub index <HTTP> {
        html_response(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Hotaru Style Guide</title>
                <style>
                    body { font-family: Arial, sans-serif; padding: 20px; }
                    h1 { color: #333; }
                    .container { max-width: 800px; margin: 0 auto; }
                </style>
            </head>
            <body>
                <div class="container">
                    <h1>Hotaru Style Guide Example</h1>
                    <p>This server demonstrates all the patterns from the Hotaru Style Guide.</p>
                    <h2>Available Endpoints:</h2>
                    <ul>
                        <li><a href="/docs">API Documentation</a></li>
                        <li><a href="/user/123">User Profile Example</a></li>
                        <li><a href="/session">Session Example</a></li>
                        <li><a href="/async">Async Example</a></li>
                    </ul>
                </div>
            </body>
            </html>
        "#)
    }
}

// Anonymous endpoint example
endpoint! {
    APP.url("/anonymous"),
    
    _ <HTTP> {
        text_response("This is an anonymous endpoint")
    }
}

// ============================================================================
// URL Pattern Matching
// ============================================================================

endpoint! {
    APP.url("/user/<int:id>"),
    
    /// # Request
    /// 
    /// `GET /user/{id}`
    /// 
    /// # Response
    /// 
    /// `JSON { "id": 123, "name": "John Doe", "email": "john@example.com" }`
    /// 
    /// # Comments
    /// 
    /// Demonstrates URL pattern matching with typed parameters
    pub get_user <HTTP> {
        let id: String = req.pattern("id").unwrap_or("0".to_string());
        
        json_response(object!({
            id: id.clone(),
            name: format!("User {}", id.clone()),
            email: format!("user{}@example.com", id.clone()),
            profile_url: format!("/user/{}/profile", id)
        }))
    }
}

endpoint! {
    APP.url("/product/<category>/<int:id>"),
    
    /// Pattern matching with multiple parameters
    pub get_product <HTTP> {
        let category: String = req.pattern("category").unwrap_or("unknown".to_string());
        let id: String = req.pattern("id").unwrap_or("0".to_string());
        
        json_response(object!({
            product_id: id.clone(),
            category: category.clone(),
            name: format!("{} Product #{}", category, id),
            price: 99.99
        }))
    }
}

// ============================================================================
// Documentation Examples
// ============================================================================

endpoint! {
    APP.url("/docs"),
    
    /// # Request
    /// 
    /// `GET /docs`
    /// 
    /// # Response
    /// 
    /// Returns HTML documentation page
    /// 
    /// # Test Commands
    /// ```bash
    /// curl http://localhost:3000/docs
    /// ```
    pub api_docs <HTTP> {
        html_response(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>API Documentation</title>
                <style>
                    body { font-family: monospace; padding: 20px; }
                    pre { background: #f0f0f0; padding: 10px; }
                </style>
            </head>
            <body>
                <h1>API Documentation</h1>
                <h2>Endpoints:</h2>
                <pre>
GET  /              - Home page
GET  /docs          - This documentation
GET  /user/{id}     - Get user by ID
POST /api/json      - Echo JSON data
GET  /session       - Session counter
                </pre>
            </body>
            </html>
        "#)
    }
}

// ============================================================================
// Async Operations
// ============================================================================

endpoint! {
    APP.url("/async"),
    
    /// Demonstrates async operations - automatically async
    pub async_example <HTTP> {
        // Simulate async work
        sleep(Duration::from_millis(100)).await;
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        json_response(object!({
            message: "Async operation completed",
            duration_ms: 100,
            timestamp: timestamp
        }))
    }
}

endpoint! {
    APP.url("/blocking"),
    
    /// Handling blocking operations properly
    pub blocking_example <HTTP> {
        use tokio::task;
        
        // Spawn blocking operation
        let result = task::spawn_blocking(|| {
            // Simulate CPU-intensive work
            std::thread::sleep(Duration::from_millis(50));
            "Blocking operation completed"
        }).await.unwrap();
        
        text_response(result)
    }
}