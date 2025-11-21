// ============================================================================
// Hotaru Style Guide - Simplified Compilable Example
// ============================================================================

// Import Conventions - Always use prelude
use hotaru::prelude::*;
use hotaru::http::*;

// ============================================================================
// Application Setup - Static APP Pattern
// ============================================================================

pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3000")
        .build()
});

// ============================================================================
// Main Entry Point
// ============================================================================

#[tokio::main]
async fn main() {
    println!("=================================================");
    println!("     Hotaru Style Guide Example Server");
    println!("=================================================");
    println!("Server running on: http://127.0.0.1:3000");
    println!("\nTest with:");
    println!("  curl http://localhost:3000/");
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
        text_response("Welcome to Hotaru Style Guide Example!")
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
    
    /// Pattern matching example
    pub get_user <HTTP> {
        let id = req.pattern("id").unwrap_or("0".to_string());
        
        json_response(object!({
            id: &id,
            name: format!("User {}", id)
        }))
    }
}

// ============================================================================
// Method Handling
// ============================================================================

endpoint! {
    APP.url("/method"),
    
    /// Different methods handling
    pub method_handler <HTTP> {
        if req.method() == POST {
            text_response("POST request")
        } else if req.method() == GET {
            text_response("GET request")
        } else {
            text_response("Other method")
        }
    }
}

// ============================================================================
// Middleware Examples
// ============================================================================

middleware! {
    /// Simple logging middleware
    pub LogRequest <HTTP> {
        println!("[LOG] Request: {} {}", req.method(), req.path());
        next(req).await
    }
}

middleware! {
    /// Short-circuit middleware example
    pub AuthCheck <HTTP> {
        if req.path() == "/protected" {
            req.response = text_response("Unauthorized");
            req
        } else {
            next(req).await
        }
    }
}

// ============================================================================
// Endpoint with Middleware
// ============================================================================

endpoint! {
    APP.url("/logged"),
    middleware = [LogRequest],
    
    /// Endpoint with middleware
    pub logged_endpoint <HTTP> {
        text_response("This request was logged")
    }
}

// ============================================================================
// Response Patterns
// ============================================================================

endpoint! {
    APP.url("/json"),
    
    /// JSON response example
    pub json_example <HTTP> {
        json_response(object!({
            status: "success",
            message: "JSON response example",
            data: {
                field1: "value1",
                field2: 42
            }
        }))
    }
}

// ============================================================================
// Form Handling
// ============================================================================

endpoint! {
    APP.url("/form"),
    
    /// Form handling example
    pub form_handler <HTTP> {
        if req.method() == POST {
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
        } else {
            text_response("Send a POST request with form data")
        }
    }
}

// ============================================================================
// Cookie Example
// ============================================================================

endpoint! {
    APP.url("/cookie"),
    
    /// Cookie handling
    pub cookie_handler <HTTP> {
        text_response("Cookie set!")
            .add_cookie("session", Cookie::new("abc123".to_string())
                .path("/")
                .max_age(3600))
    }
}

// ============================================================================
// Async Example
// ============================================================================

endpoint! {
    APP.url("/async"),
    
    /// Async operation - automatically async
    pub async_example <HTTP> {
        use tokio::time::{sleep, Duration};
        
        sleep(Duration::from_millis(100)).await;
        text_response("Async operation completed")
    }
}

// ============================================================================
// HTTP Safety Configuration
// ============================================================================

endpoint! {
    APP.url("/get-only"),
    config = [HttpSafety::new().with_allowed_method(GET)],
    
    /// GET-only endpoint
    pub get_only <HTTP> {
        text_response("This endpoint only accepts GET requests")
    }
}