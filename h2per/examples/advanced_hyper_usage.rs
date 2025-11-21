//! Example demonstrating full access to Hyper's APIs through h2per
//!
//! This example shows how you can access all of Hyper's functionality
//! including headers, body manipulation, extensions, and more.

use hotaru::prelude::*;
use h2per::prelude::*;
use h2per::hyper_exports::*;
use serde_json::json;

// Create the app with Hyper protocol
pub static APP: SApp = Lazy::new(|| {
    App::new()
        .binding("127.0.0.1:3032")
        .handle(
            HandlerBuilder::new()
                .protocol(ProtocolBuilder::new(HYPER1::new(ProtocolRole::Server)))
                .protocol(ProtocolBuilder::new(HYPER2::new(ProtocolRole::Server)))
        )
        .build()
});

#[tokio::main]
async fn main() {
    println!("Starting Advanced Hyper Example on 127.0.0.1:3032");
    println!("This example demonstrates full access to Hyper's APIs");
    APP.clone().run().await;
}

// ============================================================================
// Example 1: Direct Header Manipulation
// ============================================================================

endpoint! {
    APP.url("/headers/demo"),
    
    /// Demonstrates direct access to Hyper's HeaderMap
    pub headers_demo <HYPER1> {
        // Direct access to request headers using Hyper's HeaderMap
        let headers = req.request().headers();
        
        // Check for specific headers using Hyper's header constants
        let user_agent = headers.get(USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");
        
        let accept = headers.get(ACCEPT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("*/*");
        
        // Direct manipulation of response headers
        let response = req.response_mut();
        
        // Using Hyper's HeaderName and HeaderValue directly
        response.headers_mut().insert(
            HeaderName::from_static("x-custom-header"),
            HeaderValue::from_static("custom-value")
        );
        
        // Using predefined header constants
        response.headers_mut().insert(
            CACHE_CONTROL,
            HeaderValue::from_static("no-cache, no-store, must-revalidate")
        );
        
        response.headers_mut().insert(
            X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY")
        );
        
        response.json(json!({
            "user_agent": user_agent,
            "accept": accept,
            "custom_headers_added": true
        })).unwrap();
    }
}

// ============================================================================
// Example 2: Working with Request Extensions
// ============================================================================

endpoint! {
    APP.url("/extensions/demo"),
    
    /// Demonstrates using Hyper's Extensions for custom data
    pub extensions_demo <HYPER1> {
        // Access request extensions (custom data storage)
        let mut request = req.request_mut();
        
        // Store custom data in extensions
        #[derive(Clone, Debug)]
        struct RequestId(String);
        
        let request_id = RequestId(format!("req-{}", uuid::Uuid::new_v4()));
        request.extensions_mut().insert(request_id.clone());
        
        // Later retrieve it
        let stored_id = request.extensions()
            .get::<RequestId>()
            .map(|id| id.0.clone())
            .unwrap_or_else(|| "not-found".to_string());
        
        // Also works with response extensions
        let response = req.response_mut();
        response.extensions_mut().insert(RequestId("response-123".to_string()));
        
        response.json(json!({
            "request_id": stored_id,
            "extensions_demo": "Extensions allow storing custom typed data"
        })).unwrap();
    }
}

// ============================================================================
// Example 3: Advanced Body Handling
// ============================================================================

endpoint! {
    APP.url("/body/streaming"),
    
    /// Demonstrates streaming body responses
    pub streaming_body <HYPER1> {
        use h2per::hyper_exports::body;
        use futures::stream;
        
        // Create a streaming response
        let chunks = vec![
            Ok::<_, std::convert::Infallible>(Bytes::from("First chunk\n")),
            Ok(Bytes::from("Second chunk\n")),
            Ok(Bytes::from("Third chunk\n")),
        ];
        
        // Note: For real streaming, you'd use actual async streams
        // This is a simplified example
        let response = req.response_mut();
        
        // Set headers for streaming
        response.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8")
        );
        
        response.headers_mut().insert(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff")
        );
        
        // For now, concatenate chunks (real streaming would use a proper stream body)
        let body_content = "First chunk\nSecond chunk\nThird chunk\n";
        response.set_body(body_content.as_bytes().to_vec());
    }
}

// ============================================================================
// Example 4: HTTP Version and Protocol Details
// ============================================================================

endpoint! {
    APP.url("/protocol/info"),
    
    /// Shows detailed protocol information
    pub protocol_info <HYPER1> {
        let request = req.request();
        
        // Get HTTP version using Hyper's Version enum
        let version = request.version();
        let version_str = match version {
            Version::HTTP_09 => "HTTP/0.9",
            Version::HTTP_10 => "HTTP/1.0", 
            Version::HTTP_11 => "HTTP/1.1",
            Version::HTTP_2 => "HTTP/2.0",
            Version::HTTP_3 => "HTTP/3.0",
            _ => "Unknown",
        };
        
        // Get the full URI components
        let uri = request.uri();
        let scheme = uri.scheme_str().unwrap_or("none");
        let authority = uri.authority().map(|a| a.as_str()).unwrap_or("none");
        let path = uri.path();
        let query = uri.query().unwrap_or("none");
        
        // Get method details
        let method = request.method();
        let is_idempotent = matches!(method, &Method::GET | &Method::PUT | &Method::DELETE | &Method::HEAD);
        let is_safe = matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS);
        
        req.response_mut().json(json!({
            "version": version_str,
            "uri": {
                "scheme": scheme,
                "authority": authority,
                "path": path,
                "query": query,
                "full": uri.to_string()
            },
            "method": {
                "name": method.as_str(),
                "is_idempotent": is_idempotent,
                "is_safe": is_safe
            }
        })).unwrap();
    }
}

// ============================================================================
// Example 5: Advanced Response Building
// ============================================================================

endpoint! {
    APP.url("/response/builder"),
    
    /// Demonstrates using Hyper's Response builder pattern
    pub response_builder <HYPER1> {
        // Access the inner Hyper response directly
        let response = req.response_mut();
        
        // You can completely replace the response using Hyper's builder
        let new_response = Response::builder()
            .status(StatusCode::CREATED)
            .version(Version::HTTP_11)
            .header(CONTENT_TYPE, "application/json")
            .header(LOCATION, "/resource/123")
            .header("X-Request-Id", "12345")
            .body(body::from_string(json!({
                "created": true,
                "id": 123,
                "location": "/resource/123"
            }).to_string()))
            .unwrap();
        
        // Replace the inner response
        response.inner = new_response;
    }
}

// ============================================================================
// Example 6: Working with HTTP/2 Specific Features
// ============================================================================

endpoint! {
    APP.url("/http2/features"),
    
    /// Demonstrates HTTP/2 specific features
    pub http2_features <HYPER2> {
        // Check if we're actually using HTTP/2
        let request = req.request();
        let is_http2 = request.version() == Version::HTTP_2;
        
        // HTTP/2 supports lowercase headers and pseudo-headers
        let response = req.response_mut();
        
        if is_http2 {
            // HTTP/2 allows server push (though implementation requires more setup)
            response.headers_mut().insert(
                LINK,
                HeaderValue::from_static("</style.css>; rel=preload; as=style")
            );
        }
        
        response.json(json!({
            "is_http2": is_http2,
            "features": {
                "multiplexing": is_http2,
                "server_push": is_http2,
                "header_compression": is_http2,
                "binary_framing": is_http2
            },
            "note": "Connect with --http2-prior-knowledge flag to use HTTP/2"
        })).unwrap();
    }
}

// ============================================================================
// Example 7: Full Control with Inner Access
// ============================================================================

endpoint! {
    APP.url("/full/control"),
    
    /// Shows how to get full control over Hyper's request/response
    pub full_control <HYPER1> {
        // Get direct access to inner Hyper types
        let request_inner = req.request().as_inner();
        let response_inner = req.response_mut().as_inner_mut();
        
        // Now you have complete access to all Hyper APIs
        // You can use any method that Hyper provides
        
        // Example: Clone all request headers to response
        for (name, value) in request_inner.headers() {
            if name != CONTENT_LENGTH && name != TRANSFER_ENCODING {
                response_inner.headers_mut().insert(
                    HeaderName::from_bytes(name.as_str().as_bytes()).unwrap(),
                    value.clone()
                );
            }
        }
        
        // Set custom status
        *response_inner.status_mut() = StatusCode::ACCEPTED;
        
        // Set body using Hyper's body utilities
        *response_inner.body_mut() = body::from_string(
            "Full control over Hyper's Request and Response objects!".to_string()
        );
    }
}

// ============================================================================
// Helper endpoints for testing
// ============================================================================

endpoint! {
    APP.url("/"),
    
    /// Root endpoint with all available demos
    pub index <HYPER1> {
        req.response_mut().json(json!({
            "message": "Advanced Hyper Usage Examples",
            "endpoints": [
                {
                    "path": "/headers/demo",
                    "description": "Direct header manipulation using Hyper's HeaderMap"
                },
                {
                    "path": "/extensions/demo", 
                    "description": "Using Extensions for custom data storage"
                },
                {
                    "path": "/body/streaming",
                    "description": "Streaming body responses"
                },
                {
                    "path": "/protocol/info",
                    "description": "Detailed protocol and URI information"
                },
                {
                    "path": "/response/builder",
                    "description": "Using Hyper's Response builder pattern"
                },
                {
                    "path": "/http2/features",
                    "description": "HTTP/2 specific features (use --http2-prior-knowledge)"
                },
                {
                    "path": "/full/control",
                    "description": "Full control over Hyper request/response"
                }
            ],
            "note": "All Hyper APIs are accessible through req.request() and req.response()"
        })).unwrap();
    }
}