// ============================================================================
// Security Pattern Examples
// ============================================================================

use hotaru::prelude::*;
use hotaru::http::*;
use htmstd::Cors;
use serde_json::json;

use crate::APP;

// ============================================================================
// HTTP Safety Configuration
// ============================================================================

endpoint! {
    APP.url("/api/data"),
    config = [HttpSafety::new().with_allowed_method(GET)],
    
    /// GET-only endpoint
    pub get_only_endpoint <HTTP> {
        json_response(object!({
            status: "success",
            message: "This endpoint only accepts GET requests",
            method: "GET"
        }))
    }
}

endpoint! {
    APP.url("/api/resource"),
    config = [HttpSafety::new().with_allowed_methods(vec![GET, POST, PUT, DELETE])],
    
    /// Multi-method endpoint with explicit allowed methods
    pub resource_endpoint <HTTP> {
        match req.method() {
            &Method::GET => {
                json_response(object!({
                    status: "success",
                    action: "read",
                    message: "Resource retrieved"
                }))
            }
            &Method::POST => {
                json_response(object!({
                    status: "success",
                    action: "create",
                    message: "Resource created"
                }))
            }
            &Method::PUT => {
                json_response(object!({
                    status: "success",
                    action: "update",
                    message: "Resource updated"
                }))
            }
            &Method::DELETE => {
                json_response(object!({
                    status: "success",
                    action: "delete",
                    message: "Resource deleted"
                }))
            }
            _ => {
                // This shouldn't be reached due to HttpSafety
                json_response(object!({
                    status: "error",
                    message: "Method not allowed"
                })).status(StatusCode::METHOD_NOT_ALLOWED)
            }
        }
    }
}

// ============================================================================
// CORS Configuration
// ============================================================================

endpoint! {
    APP.url("/api/public"),
    middleware = [Cors],
    
    /// Public API endpoint with CORS enabled
    pub public_api <HTTP> {
        json_response(object!({
            status: "success",
            message: "This is a public API with CORS enabled",
            cors: "enabled",
            accessible_from: "any origin"
        }))
    }
}

endpoint! {
    APP.url("/api/restricted"),
    
    /// API endpoint without CORS (restricted to same origin)
    pub restricted_api <HTTP> {
        json_response(object!({
            status: "success",
            message: "This API is restricted to same-origin requests",
            cors: "disabled"
        }))
    }
}

// ============================================================================
// Authentication Patterns
// ============================================================================

// Simple token validation function
fn validate_token(token: &str) -> bool {
    // In real app, check against database or JWT validation
    token == "valid-token-123" || token == "admin-token-456"
}

fn get_user_role(token: &str) -> String {
    match token {
        "admin-token-456" => "admin".to_string(),
        "valid-token-123" => "user".to_string(),
        _ => "guest".to_string()
    }
}

middleware! {
    /// Authentication middleware
    pub RequireAuth <HTTP> {
        let token = req.headers()
            .get("Authorization")
            .and_then(|h| h.as_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .unwrap_or("");
        
        if !validate_token(token) {
            req.response = json_response(object!({
                status: "error",
                message: "Invalid or missing authentication token",
                hint: "Include 'Authorization: Bearer <token>' header"
            })).status(StatusCode::UNAUTHORIZED);
            req
        } else {
            // Store user info for downstream use
            let role = get_user_role(token);
            req.locals.set("user_role", role);
            req.locals.set("authenticated", true);
            next(req).await
        }
    }
}

endpoint! {
    APP.url("/api/protected"),
    middleware = [RequireAuth],
    
    /// Protected endpoint requiring authentication
    pub protected_api <HTTP> {
        let user_role = req.locals.get::<String>("user_role")
            .unwrap_or(&"unknown".to_string());
        
        json_response(object!({
            status: "success",
            message: "Access granted to protected resource",
            user_role: user_role,
            privileges: match user_role.as_str() {
                "admin" => vec!["read", "write", "delete", "manage"],
                "user" => vec!["read", "write"],
                _ => vec!["read"]
            }
        }))
    }
}

// ============================================================================
// Role-Based Access Control
// ============================================================================

middleware! {
    /// Role-based access control middleware
    pub RequireRole <HTTP> {
        // First check if authenticated
        let is_authenticated = req.locals.get::<bool>("authenticated")
            .unwrap_or(&false);
        
        if !is_authenticated {
            req.response = json_response(object!({
                status: "error",
                message: "Authentication required"
            })).status(StatusCode::UNAUTHORIZED);
            return req;
        }
        
        // Check role for admin endpoints
        if req.path().starts_with("/api/admin/") {
            let user_role = req.locals.get::<String>("user_role")
                .unwrap_or(&"guest".to_string());
            
            if user_role != "admin" {
                req.response = json_response(object!({
                    status: "error",
                    message: "Admin role required",
                    current_role: user_role
                })).status(StatusCode::FORBIDDEN);
                return req;
            }
        }
        
        next(req).await
    }
}

endpoint! {
    APP.url("/api/admin/users"),
    middleware = [RequireAuth, RequireRole],
    
    /// Admin-only endpoint
    pub admin_users <HTTP> {
        json_response(object!({
            status: "success",
            message: "Admin access granted",
            users: [
                { id: 1, name: "John Doe", role: "user" },
                { id: 2, name: "Jane Admin", role: "admin" },
                { id: 3, name: "Bob User", role: "user" }
            ]
        }))
    }
}

// ============================================================================
// Input Validation and Sanitization
// ============================================================================

endpoint! {
    APP.url("/api/search"),
    
    /// Endpoint with input validation
    pub search_endpoint <HTTP> {
        // Get query parameter
        let query = req.query_params()
            .get("q")
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        // Validate input length
        if query.is_empty() {
            return json_response(object!({
                status: "error",
                message: "Search query is required"
            })).status(StatusCode::BAD_REQUEST);
        }
        
        if query.len() > 100 {
            return json_response(object!({
                status: "error",
                message: "Search query too long (max 100 characters)"
            })).status(StatusCode::BAD_REQUEST);
        }
        
        // Check for potentially dangerous patterns
        if query.contains("<script>") || query.contains("javascript:") {
            return json_response(object!({
                status: "error",
                message: "Invalid characters in search query"
            })).status(StatusCode::BAD_REQUEST);
        }
        
        // Perform search (simulated)
        json_response(object!({
            status: "success",
            query: query,
            results: [
                { id: 1, title: format!("Result for: {}", query) },
                { id: 2, title: format!("Another match for: {}", query) }
            ],
            count: 2
        }))
    }
}

// ============================================================================
// Rate Limiting Pattern
// ============================================================================

middleware! {
    /// Simple rate limiting middleware
    pub RateLimiter <HTTP> {
        // Get client identifier (IP or user ID)
        let client_id = req.headers()
            .get("X-Forwarded-For")
            .and_then(|h| h.as_str().ok())
            .or_else(|| {
                req.headers()
                    .get("X-Real-IP")
                    .and_then(|h| h.as_str().ok())
            })
            .unwrap_or("unknown");
        
        // In real app, check against rate limit store (Redis, etc.)
        // This is a simplified example
        let is_rate_limited = false; // Would check actual rate limit
        
        if is_rate_limited {
            req.response = json_response(object!({
                status: "error",
                message: "Rate limit exceeded",
                retry_after_seconds: 60,
                limit: "100 requests per minute"
            }))
            .status(StatusCode::TOO_MANY_REQUESTS)
            .add_header("Retry-After", "60");
            req
        } else {
            // Add rate limit headers
            let mut result = next(req).await;
            result.response = result.response
                .add_header("X-RateLimit-Limit", "100")
                .add_header("X-RateLimit-Remaining", "95")
                .add_header("X-RateLimit-Reset", "1640995200");
            result
        }
    }
}

endpoint! {
    APP.url("/api/rate-limited"),
    middleware = [RateLimiter],
    
    /// Rate-limited endpoint
    pub rate_limited_api <HTTP> {
        json_response(object!({
            status: "success",
            message: "Request processed within rate limits"
        }))
    }
}

// ============================================================================
// Security Headers
// ============================================================================

middleware! {
    /// Adds security headers to responses
    pub SecurityHeaders <HTTP> {
        let mut result = next(req).await;
        
        // Add security headers
        result.response = result.response
            .add_header("X-Content-Type-Options", "nosniff")
            .add_header("X-Frame-Options", "DENY")
            .add_header("X-XSS-Protection", "1; mode=block")
            .add_header("Referrer-Policy", "strict-origin-when-cross-origin")
            .add_header("Content-Security-Policy", "default-src 'self'");
        
        result
    }
}

endpoint! {
    APP.url("/api/secure"),
    middleware = [SecurityHeaders],
    
    /// Endpoint with security headers
    pub secure_endpoint <HTTP> {
        json_response(object!({
            status: "success",
            message: "Response includes security headers",
            headers_added: [
                "X-Content-Type-Options",
                "X-Frame-Options",
                "X-XSS-Protection",
                "Referrer-Policy",
                "Content-Security-Policy"
            ]
        }))
    }
}