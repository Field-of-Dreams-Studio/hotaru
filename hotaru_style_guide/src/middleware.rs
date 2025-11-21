// ============================================================================
// Middleware Examples - Struct Style
// ============================================================================

use hotaru::prelude::*;
use hotaru::http::*;

// ============================================================================
// Basic Logging Middleware
// ============================================================================

middleware! {
    /// Logs all incoming requests
    pub LogRequest <HTTP> {
        println!("[LOG] Request: {} {}", req.method(), req.path());
        let start = std::time::Instant::now();
        
        // Continue to next middleware/endpoint
        let result = next(req).await;
        
        println!("[LOG] Response time: {:?}", start.elapsed());
        result
    }
}

// ============================================================================
// Short-Circuit Middleware
// ============================================================================

middleware! {
    /// Authentication check - stops chain if unauthorized
    pub AuthCheck <HTTP> {
        // Check for auth header
        let token = req.headers()
            .get("Authorization")
            .and_then(|h| Some(h.as_str().strip_prefix("Bearer ").unwrap_or("").to_string()))
            .unwrap_or("".to_string());
        
        if token.is_empty() || token != "valid-token-123" {
            // Don't call next() - return early with error response
            req.response = json_response(object!({
                status: "error",
                message: "Invalid or missing authentication token"
            })).status(StatusCode::UNAUTHORIZED);
            req
        } else {
            // Store user info for downstream use
            req.locals.set("user_id", "authenticated-user-123".to_string());
            next(req).await
        }
    }
}

// ============================================================================
// Data-Passing Middleware
// ============================================================================

#[derive(Clone)]
struct UserContext {
    role: String,
    permissions: Vec<String>,
}

middleware! {
    /// Sets user context for downstream middleware/endpoints
    pub SetUserContext <HTTP> {
        // Set values in locals
        req.locals.set("user_id", 123u64);
        req.locals.set("username", "john_doe".to_string());
        
        // Set complex object in params
        req.params.set(UserContext {
            role: "admin".to_string(),
            permissions: vec!["read".to_string(), "write".to_string(), "delete".to_string()]
        });
        
        println!("[MIDDLEWARE] User context set");
        next(req).await
    }
}

middleware! {
    /// Reads and uses values from upstream middleware
    pub UseUserContext <HTTP> {
        let mut result = next(req).await;
        
        // Read from locals
        let user_id = result.locals.take::<u64>("user_id").unwrap_or(0);
        let username = result.locals.take::<String>("username")
            .unwrap_or_else(|| "anonymous".to_string());
        
        // Read from params
        if let Some(context) = result.params.take::<UserContext>() {
            println!("[MIDDLEWARE] User {} (ID: {}) with role: {}", 
                     username, user_id, context.role);
        }
        
        result
    }
}

// ============================================================================
// Conditional Middleware
// ============================================================================

middleware! {
    /// Only processes requests to specific paths
    pub ConditionalMiddleware <HTTP> {
        if req.path().starts_with("/api/") {
            println!("[MIDDLEWARE] Processing API request");
            
            // Add API-specific headers
            let mut result = next(req).await;
            result.response = result.response
                .add_header("X-API-Version", "1.0")
                .add_header("X-Response-Time", "100ms");
            result
        } else {
            // Skip processing for non-API routes
            next(req).await
        }
    }
}

// ============================================================================
// Error Handling Middleware
// ============================================================================

middleware! {
    /// Catches and handles errors gracefully
    pub ErrorHandler <HTTP> {
        let path = req.path().to_owned();
        let result = next(req).await;
        
        // Check if response indicates an error
        // (In real app, you'd check status code or error flag)
        if path == "/error" {
            println!("[ERROR] Error occurred on path: {}", path);
            // Override with error response
            let mut error_result = result;
            error_result.response = json_response(object!({
                status: "error",
                message: "An error occurred processing your request",
                path: path
            })).status(StatusCode::INTERNAL_SERVER_ERROR);
            error_result
        } else {
            result
        }
    }
}

// ============================================================================
// Rate Limiting Middleware (Simplified)
// ============================================================================

middleware! {
    /// Simple rate limiting middleware
    pub RateLimit <HTTP> {
        // In real app, would check against a rate limit store
        let client_ip = req.headers()
            .get("X-Forwarded-For")
            .and_then(|h| h.as_str().ok())
            .unwrap_or("unknown");
        
        // Simulate rate limit check
        let requests_count = 5; // Would be fetched from store
        let limit = 100;
        
        if requests_count > limit {
            println!("[RATE_LIMIT] Client {} exceeded rate limit", client_ip);
            req.response = json_response(object!({
                status: "error",
                message: "Rate limit exceeded",
                retry_after: 60
            })).status(StatusCode::TOO_MANY_REQUESTS);
            req
        } else {
            println!("[RATE_LIMIT] Client {} within limits ({}/{})", 
                     client_ip, requests_count, limit);
            next(req).await
        }
    }
}