//! Response templates for Hyper - convenient response builders

pub mod response_templates {
    use crate::context::Body;
    use crate::HyperResponse;
    use hyper::{Response, StatusCode};
    use bytes::Bytes;
    use http_body_util::{Full, Empty, BodyExt};
    
    /// Create a text response
    pub fn text_response<S: Into<String>>(text: S) -> HyperResponse {
        let text = text.into();
        let body = Full::new(Bytes::from(text)).boxed();
        
        HyperResponse {
            inner: Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/plain; charset=utf-8")
                .body(body)
                .unwrap(),
        }
    }
    
    /// Create a JSON response
    pub fn json_response<T: serde::Serialize>(data: T) -> HyperResponse {
        let json = serde_json::to_vec(&data).unwrap_or_else(|e| {
            format!("{{\"error\": \"JSON serialization failed: {}\"}}", e).into_bytes()
        });
        let body = Full::new(Bytes::from(json)).boxed();
        
        HyperResponse {
            inner: Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(body)
                .unwrap(),
        }
    }
    
    /// Create an HTML response
    pub fn html_response<S: Into<String>>(html: S) -> HyperResponse {
        let html = html.into();
        let body = Full::new(Bytes::from(html)).boxed();
        
        HyperResponse {
            inner: Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/html; charset=utf-8")
                .body(body)
                .unwrap(),
        }
    }
    
    /// Create a response with custom status and body
    pub fn normal_response<B: Into<Vec<u8>>>(status: StatusCode, body: B) -> HyperResponse {
        let bytes = body.into();
        let body = Full::new(Bytes::from(bytes)).boxed();
        
        HyperResponse {
            inner: Response::builder()
                .status(status)
                .body(body)
                .unwrap(),
        }
    }
    
    /// Create a redirect response
    pub fn redirect_response<S: Into<String>>(location: S) -> HyperResponse {
        let location = location.into();
        
        HyperResponse {
            inner: Response::builder()
                .status(StatusCode::SEE_OTHER)
                .header("location", location)
                .body(Empty::<Bytes>::new().boxed())
                .unwrap(),
        }
    }
    
    /// Create a 404 Not Found response
    pub fn not_found_response() -> HyperResponse {
        HyperResponse {
            inner: Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("content-type", "text/plain; charset=utf-8")
                .body(Full::new(Bytes::from("404 Not Found")).boxed())
                .unwrap(),
        }
    }
    
    /// Create a 500 Internal Server Error response
    pub fn server_error_response<S: Into<String>>(message: S) -> HyperResponse {
        let message = message.into();
        let body = Full::new(Bytes::from(message)).boxed();
        
        HyperResponse {
            inner: Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("content-type", "text/plain; charset=utf-8")
                .body(body)
                .unwrap(),
        }
    }
    
    /// Create a 401 Unauthorized response
    pub fn unauthorized_response() -> HyperResponse {
        HyperResponse {
            inner: Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("content-type", "text/plain; charset=utf-8")
                .header("www-authenticate", "Basic realm=\"Restricted\"")
                .body(Full::new(Bytes::from("401 Unauthorized")).boxed())
                .unwrap(),
        }
    }
    
    /// Create a 403 Forbidden response
    pub fn forbidden_response() -> HyperResponse {
        HyperResponse {
            inner: Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("content-type", "text/plain; charset=utf-8")
                .body(Full::new(Bytes::from("403 Forbidden")).boxed())
                .unwrap(),
        }
    }
    
    /// Create an empty OK response
    pub fn ok_response() -> HyperResponse {
        HyperResponse {
            inner: Response::builder()
                .status(StatusCode::OK)
                .body(Empty::<Bytes>::new().boxed())
                .unwrap(),
        }
    }
}

// Extension trait for builder pattern
use crate::HyperResponse;
use hyper::StatusCode;
use http::HeaderValue;

impl HyperResponse {
    /// Add a header to the response
    pub fn with_header<K, V>(mut self, key: K, value: V) -> Self 
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        if let Ok(value) = HeaderValue::from_str(value.as_ref()) {
            self.inner.headers_mut().insert(
                http::header::HeaderName::from_bytes(key.as_ref().as_bytes()).unwrap(),
                value
            );
        }
        self
    }
    
    /// Set the status code
    pub fn with_status(mut self, status: StatusCode) -> Self {
        *self.inner.status_mut() = status;
        self
    }
}