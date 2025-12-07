// ============================================================================
// Response Pattern Examples
// ============================================================================

use hotaru::prelude::*;
use hotaru::http::*;
use serde_json::json;

use crate::APP;

// ============================================================================
// Text Response
// ============================================================================

endpoint! {
    APP.url("/response/text"),
    
    /// Simple text response
    pub text_response_example <HTTP> {
        text_response("This is a plain text response")
    }
}

// ============================================================================
// JSON Response Patterns
// ============================================================================

endpoint! {
    APP.url("/response/json/object"),
    
    /// JSON response using object! macro
    pub json_object_response <HTTP> {
        json_response(object!({
            status: "success",
            data: {
                id: 123,
                name: "John Doe",
                active: true,
                scores: [95, 87, 92]
            },
            metadata: {
                version: "1.0",
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }
        }))
    }
}

endpoint! {
    APP.url("/response/json/serde"),
    
    /// JSON response using serde_json
    pub json_serde_response <HTTP> {
        json_response(json!({
            "status": "success",
            "data": {
                "message": "Using serde_json",
                "features": ["type-safe", "fast", "standard"],
                "count": 42
            }
        }))
    }
}

// ============================================================================
// HTML Response
// ============================================================================

endpoint! {
    APP.url("/response/html"),
    
    /// HTML response example
    pub html_response_example <HTTP> {
        html_response(r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>HTML Response</title>
                <style>
                    body { 
                        font-family: system-ui, -apple-system, sans-serif;
                        max-width: 800px;
                        margin: 0 auto;
                        padding: 2rem;
                    }
                    .card {
                        border: 1px solid #ddd;
                        border-radius: 8px;
                        padding: 1rem;
                        margin: 1rem 0;
                    }
                </style>
            </head>
            <body>
                <h1>HTML Response Example</h1>
                <div class="card">
                    <h2>Features</h2>
                    <ul>
                        <li>Direct HTML rendering</li>
                        <li>Inline styles</li>
                        <li>Full HTML5 support</li>
                    </ul>
                </div>
            </body>
            </html>
        "#)
    }
}

// ============================================================================
// Custom Status Response
// ============================================================================

endpoint! {
    APP.url("/response/created"),
    
    /// Custom status code with headers
    pub created_response <HTTP> {
        let resource_id = "new-resource-123";
        let content = json!({
            "id": resource_id,
            "created_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "status": "created"
        });
        
        normal_response(StatusCode::CREATED, content.to_string().into_bytes())
            .add_header("Location", format!("/resource/{}", resource_id))
            .add_header("X-Resource-Id", resource_id)
            .add_header("Content-Type", "application/json")
    }
}

endpoint! {
    APP.url("/response/redirect"),
    
    /// Redirect response
    pub redirect_response <HTTP> {
        normal_response(StatusCode::FOUND, Vec::new())
            .add_header("Location", "/")
            .add_header("X-Redirect-Reason", "Resource moved")
    }
}

// ============================================================================
// Error Response Patterns
// ============================================================================

endpoint! {
    APP.url("/response/error/<int:code>"),
    
    /// Standardized error response format
    pub error_response <HTTP> {
        let error_code: String = req.pattern("code").unwrap_or("500".to_string());
        
        let (status, message) = match error_code.as_str() {
            "400" => (StatusCode::BAD_REQUEST, "Bad request"),
            "401" => (StatusCode::UNAUTHORIZED, "Authentication required"),
            "403" => (StatusCode::FORBIDDEN, "Access forbidden"),
            "404" => (StatusCode::NOT_FOUND, "Resource not found"),
            "500" => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error")
        };
        
        json_response(object!({
            status: "error",
            error_code: error_code,
            message: message,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            path: req.path()
        })).status(status)
    }
}

// ============================================================================
// Cookie Response
// ============================================================================

endpoint! {
    APP.url("/response/cookie"),
    
    /// Response with cookies
    pub cookie_response <HTTP> {
        text_response("Cookies have been set!")
            .add_cookie("session_id", Cookie::new("abc123xyz".to_string())
                .path("/")
                .http_only(true)
                .secure(true)
                .max_age(3600))
            .add_cookie("preferences", Cookie::new("theme=dark".to_string())
                .path("/")
                .max_age(86400 * 30)) // 30 days
    }
}

// ============================================================================
// Template Response (Akari)
// ============================================================================

endpoint! {
    APP.url("/response/template"),
    
    /// Template rendering with Akari
    pub template_response <HTTP> {
        // Note: In real app, template file would exist
        // Using akari_render! macro
        akari_render!(
            "user_profile.html",
            title = "User Profile",
            user_name = "John Doe",
            user_email = "john@example.com",
            is_admin = true,
            permissions = ["read", "write", "delete"],
            last_login = "2024-01-01 12:00:00"
        )
    }
}

endpoint! {
    APP.url("/response/plain-template"),
    
    /// Plain template without data
    pub plain_template_example <HTTP> {
        plain_template_response("static_page.html")
    }
}

// ============================================================================
// File Download Response
// ============================================================================

endpoint! {
    APP.url("/response/download"),
    
    /// File download with proper headers
    pub download_response <HTTP> {
        let file_content = b"This is the content of the downloaded file";
        let filename = "example.txt";
        
        normal_response(StatusCode::OK, file_content.to_vec())
            .add_header("Content-Type", "application/octet-stream")
            .add_header("Content-Disposition", 
                       format!("attachment; filename=\"{}\"", filename))
            .add_header("Content-Length", file_content.len().to_string())
    }
}