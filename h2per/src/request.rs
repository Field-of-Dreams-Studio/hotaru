//! Request templates for Hyper - convenient request builders

pub mod request_templates {
    use crate::context::Body;
    use hyper::{Request, Method};
    use bytes::Bytes;
    use http_body_util::{Full, Empty, BodyExt};
    use std::collections::HashMap;
    
    /// Create a GET request
    pub fn get<S: Into<String>>(uri: S) -> Request<Body> {
        Request::builder()
            .method(Method::GET)
            .uri(uri.into())
            .header("user-agent", "h2per/0.1.0")
            .body(Empty::<Bytes>::new().boxed())
            .unwrap()
    }
    
    /// Create a POST request with JSON body
    pub fn json_post<S: Into<String>, T: serde::Serialize>(uri: S, data: T) -> Request<Body> {
        let json = serde_json::to_vec(&data).unwrap();
        let body = Full::new(Bytes::from(json)).boxed();
        
        Request::builder()
            .method(Method::POST)
            .uri(uri.into())
            .header("content-type", "application/json")
            .header("user-agent", "h2per/0.1.0")
            .body(body)
            .unwrap()
    }
    
    /// Create a POST request with form data
    pub fn form_post<S: Into<String>>(uri: S, form_data: HashMap<String, String>) -> Request<Body> {
        let encoded = url_encode_form(&form_data);
        let body = Full::new(Bytes::from(encoded)).boxed();
        
        Request::builder()
            .method(Method::POST)
            .uri(uri.into())
            .header("content-type", "application/x-www-form-urlencoded")
            .header("user-agent", "h2per/0.1.0")
            .body(body)
            .unwrap()
    }
    
    /// Create a PUT request with JSON body
    pub fn json_put<S: Into<String>, T: serde::Serialize>(uri: S, data: T) -> Request<Body> {
        let json = serde_json::to_vec(&data).unwrap();
        let body = Full::new(Bytes::from(json)).boxed();
        
        Request::builder()
            .method(Method::PUT)
            .uri(uri.into())
            .header("content-type", "application/json")
            .header("user-agent", "h2per/0.1.0")
            .body(body)
            .unwrap()
    }
    
    /// Create a DELETE request
    pub fn delete<S: Into<String>>(uri: S) -> Request<Body> {
        Request::builder()
            .method(Method::DELETE)
            .uri(uri.into())
            .header("user-agent", "h2per/0.1.0")
            .body(Empty::<Bytes>::new().boxed())
            .unwrap()
    }
    
    /// Create a PATCH request with JSON body
    pub fn json_patch<S: Into<String>, T: serde::Serialize>(uri: S, data: T) -> Request<Body> {
        let json = serde_json::to_vec(&data).unwrap();
        let body = Full::new(Bytes::from(json)).boxed();
        
        Request::builder()
            .method(Method::PATCH)
            .uri(uri.into())
            .header("content-type", "application/json")
            .header("user-agent", "h2per/0.1.0")
            .body(body)
            .unwrap()
    }
    
    /// Create a HEAD request
    pub fn head<S: Into<String>>(uri: S) -> Request<Body> {
        Request::builder()
            .method(Method::HEAD)
            .uri(uri.into())
            .header("user-agent", "h2per/0.1.0")
            .body(Empty::<Bytes>::new().boxed())
            .unwrap()
    }
    
    /// Create a request with custom method and body
    pub fn custom_request<S: Into<String>, B: Into<Vec<u8>>>(
        method: Method,
        uri: S,
        body: B
    ) -> Request<Body> {
        let bytes = body.into();
        let body = if bytes.is_empty() {
            Empty::<Bytes>::new().boxed()
        } else {
            Full::new(Bytes::from(bytes)).boxed()
        };
        
        Request::builder()
            .method(method)
            .uri(uri.into())
            .header("user-agent", "h2per/0.1.0")
            .body(body)
            .unwrap()
    }
    
    // Helper function to URL encode form data
    fn url_encode_form(data: &HashMap<String, String>) -> String {
        data.iter()
            .map(|(k, v)| {
                format!("{}={}", 
                    urlencoding::encode(k),
                    urlencoding::encode(v))
            })
            .collect::<Vec<_>>()
            .join("&")
    }
}

// Extension trait for request builder pattern
use crate::context::Body;
use hyper::Request;
use http::HeaderValue;

pub trait RequestExt {
    fn with_header<K, V>(self, key: K, value: V) -> Self
    where
        K: AsRef<str>,
        V: AsRef<str>;
    
    fn with_bearer_auth<S: AsRef<str>>(self, token: S) -> Self;
    
    fn with_basic_auth<U: AsRef<str>, P: AsRef<str>>(self, username: U, password: P) -> Self;
}

impl RequestExt for Request<Body> {
    fn with_header<K, V>(mut self, key: K, value: V) -> Self 
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        if let Ok(value) = HeaderValue::from_str(value.as_ref()) {
            self.headers_mut().insert(
                http::header::HeaderName::from_bytes(key.as_ref().as_bytes()).unwrap(),
                value
            );
        }
        self
    }
    
    fn with_bearer_auth<S: AsRef<str>>(self, token: S) -> Self {
        self.with_header("authorization", format!("Bearer {}", token.as_ref()))
    }
    
    fn with_basic_auth<U: AsRef<str>, P: AsRef<str>>(self, username: U, password: P) -> Self {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let credentials = format!("{}:{}", username.as_ref(), password.as_ref());
        let encoded = STANDARD.encode(credentials);
        self.with_header("authorization", format!("Basic {}", encoded))
    }
}