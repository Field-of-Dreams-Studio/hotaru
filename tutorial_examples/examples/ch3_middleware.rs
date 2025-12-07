//! Chapter 3: Middleware - The Power of Composition
//! 
//! This example demonstrates Hotaru's middleware system including
//! global middleware, endpoint-specific middleware, and the .. pattern

use hotaru::prelude::*;
use hotaru::http::*;
use htmstd::cors::cors::CorsMiddleware;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

// Define middleware

// Simple logging middleware
middleware! {
    /// Logs all incoming requests
    pub request_logger {
        println!("[LOG] {} {} from {}", 
            req.method(), 
            req.path(),
            req.header("User-Agent").string()
        );
        next.run(req).await
    }
}

// Request timing middleware
middleware! {
    /// Measures request processing time
    pub timing_middleware {
        let start = Instant::now();
        let response = next.run(req).await;
        let duration = start.elapsed();
        
        println!("[TIMER] Request to {} took {:?}", req.path(), duration);
        
        // Add timing header to response
        let mut response = response;
        response.headers_mut().insert(
            "X-Response-Time",
            format!("{}ms", duration.as_millis()).parse().unwrap()
        );
        response
    }
}

// Authentication middleware
middleware! {
    /// Checks for valid authorization token
    pub auth_check {
        let token = req.header("Authorization").string();
        
        // Simple token validation (in real app, verify JWT or session)
        if token.is_empty() || !token.starts_with("Bearer ") {
            let mut response = HttpResponse::default();
            response = response.status(StatusCode::UNAUTHORIZED)
                .add_header("WWW-Authenticate", "Bearer");
            response.body = "Unauthorized: Missing or invalid token".into();
            return response;
        }
        
        // Extract token value and validate
        let token_value = &token[7..]; // Skip "Bearer "
        if token_value != "secret-token-123" {
            let mut response = HttpResponse::default();
            response = response.status(StatusCode::FORBIDDEN);
            response.body = "Forbidden: Invalid token".into();
            return response;
        }
        
        println!("[AUTH] Valid token provided");
        next.run(req).await
    }
}

// Rate limiting middleware
middleware! {
    /// Simple rate limiter (1 request per second)
    pub rate_limiter {
        static LAST_REQUEST: Lazy<Mutex<Instant>> = 
            Lazy::new(|| Mutex::new(Instant::now()));
        
        let mut last = LAST_REQUEST.lock().unwrap();
        let now = Instant::now();
        
        if now.duration_since(*last).as_secs() < 1 {
            let mut response = HttpResponse::default();
            response = response.status(StatusCode::TOO_MANY_REQUESTS)
                .add_header("Retry-After", "1");
            response.body = "Too many requests. Please wait 1 second.".into();
            return response;
        }
        
        *last = now;
        drop(last);
        
        next.run(req).await
    }
}

// Request counter middleware (stateful)
middleware! {
    /// Counts total requests and adds count to response headers
    pub request_counter {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        
        let count = COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
        println!("[COUNTER] Request #{}", count);
        
        let mut response = next.run(req).await;
        response.headers_mut().insert(
            "X-Request-Count",
            count.to_string().parse().unwrap()
        );
        response
    }
}

// Define application with global middleware
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3002")
        // Global middleware applied to all endpoints by default
        .middleware(request_logger)
        .middleware(timing_middleware)
        .middleware(CorsMiddleware::default())
        .build()
});

// Public endpoint - uses only global middleware (automatic inheritance)
endpoint! {
    APP.url("/"),
    
    /// Home endpoint with automatic global middleware
    pub home <HTTP> {
        text_response("Welcome! This endpoint uses global middleware automatically.")
    }
}

// Public endpoint with additional middleware
endpoint! {
    APP.url("/counted"),
    middleware = [.., request_counter],  // .. means "inherit global middleware"
    
    /// Endpoint that counts requests
    pub counted_endpoint <HTTP> {
        text_response("Check the X-Request-Count header!")
    }
}

// Protected endpoint with auth middleware
endpoint! {
    APP.url("/api/protected"),
    middleware = [.., auth_check],  // Global middleware + auth
    
    /// Protected endpoint requiring authentication
    pub protected_endpoint <HTTP> {
        json_response(object!({
            message: "Welcome to the protected area!",
            user: "authenticated_user"
        }))
    }
}

// Admin endpoint with multiple additional middleware
endpoint! {
    APP.url("/api/admin"),
    middleware = [.., auth_check, rate_limiter],  // Global + auth + rate limit
    
    /// Admin endpoint with authentication and rate limiting
    pub admin_endpoint <HTTP> {
        json_response(object!({
            message: "Admin access granted",
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }))
    }
}

// Health check endpoint - explicitly no middleware
endpoint! {
    APP.url("/health"),
    middleware = [],  // Empty array = no middleware at all
    
    /// Health check endpoint without any middleware
    pub health_check <HTTP> {
        text_response("OK")
    }
}

// Custom middleware chain without global inheritance
endpoint! {
    APP.url("/custom"),
    middleware = [request_counter],  // Only request_counter, no global middleware
    
    /// Endpoint with custom middleware chain (no global inheritance)
    pub custom_middleware <HTTP> {
        text_response("This endpoint only uses request_counter middleware")
    }
}

// Conditional middleware example
middleware! {
    /// Only logs POST requests
    pub post_logger {
        if req.method() == HttpMethod::POST {
            println!("[POST-LOG] POST request to {}", req.path());
            
            // Log request body if JSON
            if let Some(json) = req.json().await {
                println!("[POST-LOG] Body: {:?}", json);
            }
        }
        next.run(req).await
    }
}

endpoint! {
    APP.url("/api/data"),
    middleware = [.., post_logger],
    
    /// Endpoint that logs POST requests specially
    pub data_endpoint <HTTP> {
        match req.method() {
            HttpMethod::GET => {
                json_response(object!({
                    message: "GET request received"
                }))
            },
            HttpMethod::POST => {
                json_response(object!({
                    message: "POST request received and logged"
                }))
            },
            _ => {
                let mut response = HttpResponse::default();
                response = response.status(StatusCode::METHOD_NOT_ALLOWED);
                response.body = "Method not allowed".into();
                response
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("\n🚀 Chapter 3: Middleware System");
    println!("================================");
    println!("Server running at http://127.0.0.1:3002\n");
    
    println!("Global middleware (applied to most endpoints):");
    println!("  - request_logger");
    println!("  - timing_middleware");
    println!("  - CORS\n");
    
    println!("Endpoints:");
    println!("  GET /              - Uses global middleware (automatic)");
    println!("  GET /counted       - Global + request counter");
    println!("  GET /api/protected - Global + auth (needs token)");
    println!("  GET /api/admin     - Global + auth + rate limit");
    println!("  GET /health        - No middleware at all");
    println!("  GET /custom        - Only request counter (no global)");
    println!("  *   /api/data      - Global + POST logger\n");
    
    println!("Test commands:");
    println!("  # Basic request (see logs)");
    println!("  curl -i http://localhost:3002/\n");
    
    println!("  # See request counter");
    println!("  curl -i http://localhost:3002/counted\n");
    
    println!("  # Try protected without token (401)");
    println!("  curl -i http://localhost:3002/api/protected\n");
    
    println!("  # Access protected with token");
    println!("  curl -i -H 'Authorization: Bearer secret-token-123' \\");
    println!("       http://localhost:3002/api/protected\n");
    
    println!("  # Test rate limiting (run twice quickly)");
    println!("  curl -H 'Authorization: Bearer secret-token-123' \\");
    println!("       http://localhost:3002/api/admin\n");
    
    println!("  # Health check (no middleware, no logs)");
    println!("  curl http://localhost:3002/health\n");
    
    println!("  # POST request (special logging)");
    println!("  curl -X POST http://localhost:3002/api/data\n");
    
    println!("Watch the console for middleware logs!\n");
    
    APP.clone().run().await;
}