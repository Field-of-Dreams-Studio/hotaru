use hotaru::prelude::*; 
use hotaru::http::*;  
use htmstd::{CookieSession, Cors};

use crate::APP;

// ============================================================================
// Session middleware examples
// ============================================================================

endpoint! {
    APP.url("/session/test"),
    middleware = [CookieSession],
    
    /// Session test endpoint
    pub session_test <HTTP> {
        let session = req.params.get_mut::<htmstd::session::cookie_session::CSessionRW>().unwrap();
        // Get count value as string from session, default to "0"
        let count_val = session.get("count").cloned().unwrap_or(Value::new("0"));
        let count_str = count_val.to_string();
        let count: i32 = count_str.parse().unwrap_or(0) + 1;
        session.insert("count".to_string(), Value::new(count.to_string()));
        text_response(format!("Visit count: {}", count))
    }
}

// ============================================================================
// CORS middleware example
// ============================================================================

endpoint! {
    APP.url("/api/data"),
    middleware = [Cors],
    
    /// API endpoint with CORS
    pub api_data <HTTP> {
        let message = "This endpoint has CORS enabled";
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        json_response(object!({
            message: message,
            timestamp: timestamp
        }))
    }
}

endpoint! {
    APP.url("/api/protected"),
    middleware = [CookieSession, Cors],
    
    /// Protected API endpoint with session and CORS
    pub api_protected <HTTP> {
        let session = req.params.get_mut::<htmstd::session::cookie_session::CSessionRW>().unwrap();
        
        // Check if user is authenticated  
        let is_authenticated = if let Some(v) = session.get("authenticated") {
            v.to_string() == "true"
        } else {
            false
        };
        
        if is_authenticated {
            json_response(object!({
                status: "success",
                message: "You have access to protected data",
                data: {
                    secret: "This is protected information",
                    level: "confidential"
                }
            }))
        } else {
            json_response(object!({
                status: "error",
                message: "Authentication required"
            }))
        }
    }
}

endpoint! {
    APP.url("/api/login"),
    middleware = [CookieSession],
    
    /// Login endpoint
    pub api_login <HTTP> {
        if req.method() == POST {
            match req.json().await {
                Some(json) => {
                    let username = json.get("username").to_string();
                    let password = json.get("password").to_string();
                    
                    // Simple authentication check (in real app, verify against database)
                    if username == "\"admin\"" && password == "\"password\"" {
                        let session = req.params.get_mut::<htmstd::session::cookie_session::CSessionRW>().unwrap();
                        session.insert("authenticated".to_string(), Value::new("true"));
                        session.insert("username".to_string(), Value::new(username));
                        
                        json_response(object!({
                            status: "success",
                            message: "Login successful"
                        }))
                    } else {
                        json_response(object!({
                            status: "error",
                            message: "Invalid credentials"
                        }))
                    }
                }
                None => {
                    json_response(object!({
                        status: "error",
                        message: "Invalid JSON data"
                    }))
                }
            }
        } else {
            json_response(object!({
                status: "error",
                message: "Method not allowed"
            }))
        }
    }
}