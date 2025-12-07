//! Chapter 1: Getting Started - Your First Hotaru Server
//! 
//! This example demonstrates the basics of Hotaru from the QUICK_TUTORIAL.md

use hotaru::prelude::*;
use hotaru::http::*;
use tutorial_examples::{text_response, json_response};

// Define your application with a static binding
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3000")
        .build()
});

// The simplest possible endpoint
endpoint! {
    APP.url("/"),
    
    /// Root endpoint - returns a simple greeting
    pub root <HTTP> {
        text_response("Welcome to Hotaru!")
    }
}

// Path parameters example
endpoint! {
    APP.url("/hello/{name}"),
    
    /// Greet a user by name
    /// Path parameter {name} is automatically extracted
    pub greet <HTTP> {
        let name = req.param("name").as_str();
        text_response(format!("Hello, {}!", name))
    }
}

// Understanding contexts - this shows what's available in req
endpoint! {
    APP.url("/context-demo"),
    
    /// Demonstrates the HttpContext capabilities
    pub context_demo <HTTP> {
        // The context provides these conveniences:
        let method = req.method();           // HTTP method
        let path = req.path();               // Request path
        let user_agent = req.header("User-Agent").as_str();
        
        // You can also access the full request if needed
        let _headers = &req.request.meta.header;
        
        json_response(object!({
            message: "Context demonstration",
            method: method.to_string(),
            path: path,
            user_agent: user_agent
        }))
    }
}

// Query parameters example
endpoint! {
    APP.url("/search"),
    
    /// Search endpoint with query parameters
    /// Example: /search?q=rust&limit=10
    pub search <HTTP> {
        // Note: req.query() returns Option<&Value>
        let query = req.query("q")
            .map(|v| v.as_str())
            .unwrap_or("");
        
        let limit = req.query("limit")
            .and_then(|v| v.as_str().parse::<u32>().ok())
            .unwrap_or(10);
        
        json_response(object!({
            query: query,
            limit: limit,
            results: [
                "Result 1",
                "Result 2", 
                "Result 3"
            ]
        }))
    }
}

// Multiple methods on same endpoint
endpoint! {
    APP.url("/api/data"),
    
    /// Handles different HTTP methods
    pub api_data <HTTP> {
        match req.method() {
            HttpMethod::GET => {
                json_response(object!({
                    message: "Fetching data",
                    method: "GET"
                }))
            },
            HttpMethod::POST => {
                json_response(object!({
                    message: "Creating data",
                    method: "POST"
                }))
            },
            HttpMethod::DELETE => {
                json_response(object!({
                    message: "Deleting data",
                    method: "DELETE"
                }))
            },
            _ => {
                // For unsupported methods, return 405 Method Not Allowed
                let mut response = HttpResponse::default();
                response = response.status(StatusCode::METHOD_NOT_ALLOWED)
                    .add_header("Allow", "GET, POST, DELETE");
                response.body = HttpBody::Text("Method Not Allowed".into());
                response
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("\n🚀 Chapter 1: Basic Hotaru Server");
    println!("==================================");
    println!("Server running at http://127.0.0.1:3000\n");
    
    println!("Try these endpoints:");
    println!("  GET  /                       - Welcome message");
    println!("  GET  /hello/World            - Personalized greeting");
    println!("  GET  /context-demo           - See context information");
    println!("  GET  /search?q=rust&limit=5  - Query parameters");
    println!("  GET  /api/data               - GET method");
    println!("  POST /api/data               - POST method\n");
    
    println!("Test with curl:");
    println!("  curl http://localhost:3000/");
    println!("  curl http://localhost:3000/hello/Rustacean");
    println!("  curl http://localhost:3000/search?q=hotaru");
    println!("  curl -X POST http://localhost:3000/api/data\n");
    
    APP.clone().run().await;
}