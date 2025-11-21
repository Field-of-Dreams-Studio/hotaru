// ============================================================================
// Request Handling Examples
// ============================================================================

use hotaru::prelude::*;
use hotaru::http::*;
use htmstd::CookieSession;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::APP;

// ============================================================================
// Method Checking
// ============================================================================

endpoint! {
    APP.url("/request/method"),
    
    /// Handles different HTTP methods
    pub method_handler <HTTP> {
        if req.method() == POST {
            json_response(object!({
                status: "success",
                message: "POST request received"
            }))
        } else if req.method() == GET {
            json_response(object!({
                status: "success",
                message: "GET request received"
            }))
        } else if req.method() == PUT {
            json_response(object!({
                status: "success",
                message: "PUT request received"
            }))
        } else {
            json_response(object!({
                status: "success",
                message: format!("{} request received", req.method())
            }))
        }
    }
}

// ============================================================================
// Form Data Handling
// ============================================================================

endpoint! {
    APP.url("/request/form"),
    
    /// Handles form submissions
    pub form_handler <HTTP> {
        if req.method() == GET {
            // Return form HTML
            html_response(r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Form Example</title>
                    <style>
                        body { font-family: Arial; padding: 20px; }
                        input, button { display: block; margin: 10px 0; padding: 5px; }
                    </style>
                </head>
                <body>
                    <h1>Form Example</h1>
                    <form method="POST" action="/request/form">
                        <input type="text" name="username" placeholder="Username" required>
                        <input type="email" name="email" placeholder="Email" required>
                        <input type="number" name="age" placeholder="Age" required>
                        <button type="submit">Submit</button>
                    </form>
                </body>
                </html>
            "#)
        } else {
            // Handle POST
            match req.form().await {
                Some(form) => {
                    let username = form.data.get("username")
                        .map(|s| s.as_str())
                        .unwrap_or("anonymous");
                    let email = form.data.get("email")
                        .map(|s| s.as_str())
                        .unwrap_or("no-email");
                    let age = form.data.get("age")
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(0);
                    
                    json_response(object!({
                        status: "success",
                        message: "Form data received",
                        data: {
                            username: username,
                            email: email,
                            age: age
                        }
                    }))
                }
                None => {
                    json_response(object!({
                        status: "error",
                        message: "Invalid form data"
                    })).status(StatusCode::BAD_REQUEST)
                }
            }
        }
    }
}

// ============================================================================
// JSON Data Handling
// ============================================================================

#[derive(Deserialize, Serialize)]
struct UserInput {
    name: String,
    email: String,
    age: u32,
    tags: Vec<String>,
}

endpoint! {
    APP.url("/api/json"),
    config = [HttpSafety::new().with_allowed_method(POST)],
    
    /// Generic JSON handling
    pub json_handler <HTTP> {
        match req.json::<serde_json::Value>().await {
            Some(json_data) => {
                // Echo back the received JSON
                json_response(object!({
                    status: "success",
                    message: "JSON received",
                    received_data: json_data,
                    processed_at: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                }))
            }
            None => {
                json_response(object!({
                    status: "error",
                    message: "Invalid or missing JSON data"
                })).status(StatusCode::BAD_REQUEST)
            }
        }
    }
}

endpoint! {
    APP.url("/api/typed-json"),
    config = [HttpSafety::new().with_allowed_method(POST)],
    
    /// Typed JSON handling
    pub typed_json_handler <HTTP> {
        match req.json::<UserInput>().await {
            Some(user_data) => {
                // Process typed data
                json_response(object!({
                    status: "success",
                    message: format!("User {} registered", user_data.name),
                    user: {
                        name: user_data.name,
                        email: user_data.email,
                        age: user_data.age,
                        tag_count: user_data.tags.len()
                    }
                }))
            }
            None => {
                json_response(object!({
                    status: "error",
                    message: "Invalid user data format",
                    expected_format: {
                        name: "string",
                        email: "string",
                        age: "number",
                        tags: ["array", "of", "strings"]
                    }
                })).status(StatusCode::BAD_REQUEST)
            }
        }
    }
}

// ============================================================================
// Cookie Management
// ============================================================================

endpoint! {
    APP.url("/request/cookies"),
    
    /// Cookie reading and setting
    pub cookie_handler <HTTP> {
        if req.method() == GET {
            // Read cookies
            let cookies = req.get_cookies();
            let mut cookie_list = Vec::new();
            
            for (name, cookie) in cookies.0.iter() {
                cookie_list.push(object!({
                    name: name,
                    value: cookie.get_value()
                }));
            }
            
            json_response(object!({
                status: "success",
                cookies: cookie_list,
                count: cookie_list.len()
            }))
        } else {
            // Set new cookie
            let cookie_name = "test_cookie";
            let cookie_value = format!("value_{}", 
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs());
            
            json_response(object!({
                status: "success",
                message: "Cookie set",
                cookie: {
                    name: cookie_name,
                    value: &cookie_value
                }
            })).add_cookie(cookie_name, Cookie::new(cookie_value)
                .path("/")
                .http_only(true)
                .max_age(3600))
        }
    }
}

// ============================================================================
// Session Handling
// ============================================================================

endpoint! {
    APP.url("/session"),
    middleware = [CookieSession],
    
    /// Session counter example
    pub session_handler <HTTP> {
        let session = req.params.get_mut::<htmstd::session::cookie_session::CSessionRW>()
            .unwrap();
        
        // Get visit count from session
        let count = session.get("visit_count")
            .cloned()
            .unwrap_or(Value::new("0"))
            .to_string()
            .parse::<i32>()
            .unwrap_or(0);
        
        // Increment and save
        let new_count = count + 1;
        session.insert("visit_count".to_string(), 
                      Value::new(new_count.to_string()));
        
        // Get session ID (if available)
        let session_id = session.get("session_id")
            .cloned()
            .unwrap_or(Value::new("new-session"));
        
        if count == 0 {
            // First visit - set session ID
            let new_session_id = format!("session_{}", 
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs());
            session.insert("session_id".to_string(), 
                          Value::new(new_session_id.clone()));
            
            json_response(object!({
                status: "success",
                message: "Welcome! New session created",
                session_id: new_session_id,
                visit_count: new_count
            }))
        } else {
            json_response(object!({
                status: "success",
                message: format!("Welcome back! Visit #{}", new_count),
                session_id: session_id.to_string(),
                visit_count: new_count
            }))
        }
    }
}

// ============================================================================
// Headers and Query Parameters
// ============================================================================

endpoint! {
    APP.url("/request/headers"),
    
    /// Access request headers
    pub headers_handler <HTTP> {
        let mut headers_list = Vec::new();
        
        for (name, value) in req.headers().iter() {
            if let Ok(v) = value.as_str() {
                headers_list.push(object!({
                    name: name.as_str(),
                    value: v
                }));
            }
        }
        
        // Access specific headers
        let user_agent = req.headers()
            .get("User-Agent")
            .and_then(|h| h.as_str().ok())
            .unwrap_or("Unknown");
        
        let content_type = req.headers()
            .get("Content-Type")
            .and_then(|h| h.as_str().ok())
            .unwrap_or("Not specified");
        
        json_response(object!({
            status: "success",
            user_agent: user_agent,
            content_type: content_type,
            all_headers: headers_list,
            header_count: headers_list.len()
        }))
    }
}

// ============================================================================
// Error Handling in Requests
// ============================================================================

endpoint! {
    APP.url("/request/safe/<action>"),
    
    /// Demonstrates graceful error handling
    pub safe_handler <HTTP> {
        let action = req.pattern("action").unwrap_or("default".to_string());
        
        match action.as_str() {
            "success" => {
                json_response(object!({
                    status: "success",
                    message: "Operation completed successfully"
                }))
            }
            "error" => {
                json_response(object!({
                    status: "error",
                    message: "Simulated error condition"
                })).status(StatusCode::INTERNAL_SERVER_ERROR)
            }
            "not-found" => {
                json_response(object!({
                    status: "error",
                    message: "Resource not found",
                    resource: action
                })).status(StatusCode::NOT_FOUND)
            }
            _ => {
                // Graceful fallback
                json_response(object!({
                    status: "success",
                    message: "Unknown action - using default",
                    requested_action: action,
                    fallback: "default"
                }))
            }
        }
    }
}