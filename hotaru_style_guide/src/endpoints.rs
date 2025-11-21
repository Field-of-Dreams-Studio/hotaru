// ============================================================================
// Endpoint Examples
// ============================================================================

use hotaru::prelude::*;
use hotaru::http::*;
use serde_json::json;

use crate::APP;
use crate::middleware::{LogRequest, AuthCheck, SetUserContext};

// ============================================================================
// Endpoint with Middleware
// ============================================================================

endpoint! {
    APP.url("/logged"),
    middleware = [LogRequest],
    
    /// Endpoint with logging middleware
    pub logged_endpoint <HTTP> {
        text_response("This request was logged")
    }
}

endpoint! {
    APP.url("/protected"),
    middleware = [AuthCheck, LogRequest],
    
    /// Protected endpoint requiring authentication
    pub protected_endpoint <HTTP> {
        json_response(object!({
            status: "success",
            message: "You have accessed a protected resource",
            user_id: req.locals.get::<String>("user_id").unwrap_or(&"unknown".to_string())
        }))
    }
}

// ============================================================================
// Endpoint with Configuration
// ============================================================================

endpoint! {
    APP.url("/api/readonly"),
    config = [HttpSafety::new().with_allowed_method(GET)],
    
    /// GET-only endpoint
    pub readonly_api <HTTP> {
        json_response(object!({
            status: "success",
            message: "This endpoint only accepts GET requests"
        }))
    }
}

endpoint! {
    APP.url("/api/write"),
    config = [HttpSafety::new().with_allowed_methods(vec![POST, PUT])],
    
    /// Write-only endpoint (POST/PUT)
    pub write_api <HTTP> {
        let method = req.method().to_string();
        json_response(object!({
            status: "success",
            message: format!("Write operation via {}", method)
        }))
    }
}

// ============================================================================
// Complex endpoint with everything
// ============================================================================

endpoint! {
    APP.url("/api/complex/<int:id>"),
    middleware = [SetUserContext, LogRequest],
    config = [HttpSafety::new().with_allowed_methods(vec![GET, POST])],
    
    /// # Request
    /// 
    /// `GET/POST /api/complex/{id}`
    /// 
    /// # Response
    /// 
    /// ```json
    /// {
    ///   "status": "success",
    ///   "id": 123,
    ///   "method": "GET",
    ///   "user_context": "..."
    /// }
    /// ```
    pub complex_endpoint <HTTP> {
        let id: String = req.pattern("id").unwrap_or("0".to_string());
        let method = req.method().to_string();
        
        // Access middleware-set values
        let user_id = req.locals.get::<u64>("user_id").unwrap_or(&0);
        
        json_response(object!({
            status: "success",
            id: id,
            method: method,
            user_id: user_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }))
    }
}