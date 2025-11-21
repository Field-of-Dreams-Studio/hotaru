//! Unified context for Hyper-based HTTP protocols.
//! Maintains API compatibility with Hotaru's HttpContext while using Hyper internally.

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::any::Any;
use serde::de::DeserializeOwned;

use akari::{
    extensions::{Params, Locals, ParamsClone, LocalsClone},
    Value,
};

use hotaru_core::{
    connection::{RequestContext, ProtocolRole, ConnectionStatus},
    app::application::App,
    url::Url,
    http::form::UrlEncodedForm,
};

use hyper::{Request, Response, Method, StatusCode, Version};
use http::{HeaderMap, HeaderName, HeaderValue};
use bytes::Bytes;
use http_body_util::{Full, Empty, BodyExt, combinators::BoxBody};

// ============================================================================
// HyperContext - Unified context for all HTTP versions
// ============================================================================

/// Unified context that works with HTTP/1, HTTP/2, and HTTP/3 via Hyper.
pub struct HyperContext {
    /// HTTP version being used
    pub version: HttpVersion,
    
    /// The incoming request
    pub request: HyperRequest,
    
    /// The outgoing response
    pub response: HyperResponse,
    
    /// Request parameters (path params, query params, etc.)
    pub params: RwLock<Params>,
    
    /// Thread-local storage for middleware and handlers
    pub locals: RwLock<Locals>,
    
    /// Reference to the application
    pub app: Option<Arc<App>>,
    
    /// Template manager for rendering (if configured)
    pub template_manager: Option<Arc<akari::TemplateManager>>,
    
    /// Stream ID (for HTTP/2 and HTTP/3)
    pub stream_id: Option<u32>,
    
    /// Protocol role
    role: ProtocolRole,
    
    /// The matched endpoint URL (for parameter extraction)
    pub endpoint: Option<Arc<Url<Self>>>,
    
    /// Path segments from the URL
    pub path_segments: Vec<String>,
    
    /// Connection status for signaling protocol switches
    pub connection_status: ConnectionStatus,
    
    /// HTTP-specific upgrade context (if upgrade is in progress)
    pub upgrade_context: Option<crate::upgrade::UpgradeContext>,
    
    /// Target protocol for upgrade (using HTTP-specific enum)
    pub upgrade_target: Option<crate::upgrade::HttpProtocol>,
}

#[derive(Clone, Debug)]
pub enum HttpVersion {
    Http1_0,
    Http1_1,
    Http2,
    Http3,
}

/// Type for request/response bodies
pub type Body = BoxBody<Bytes, std::convert::Infallible>;

/// Wrapper around Hyper's Request with convenience methods
pub struct HyperRequest {
    /// Direct access to Hyper's Request - all Hyper APIs available
    pub inner: Request<Body>,
    pub path_params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body_bytes: Option<Vec<u8>>,  // Store body bytes for form/json parsing
}

/// Wrapper around Hyper's Response with convenience methods  
pub struct HyperResponse {
    /// Direct access to Hyper's Response - all Hyper APIs available
    pub inner: Response<Body>,
}

impl HyperContext {
    /// Create a new context for server handling
    pub fn new_server(
        request: Request<Body>,
        app: Arc<App>,
    ) -> Self {
        let version = match request.version() {
            Version::HTTP_09 | Version::HTTP_10 => HttpVersion::Http1_0,
            Version::HTTP_11 => HttpVersion::Http1_1,
            Version::HTTP_2 => HttpVersion::Http2,
            Version::HTTP_3 => HttpVersion::Http3,
            _ => HttpVersion::Http1_1,
        };
        
        let mut params = Params::new();
        let locals = Locals::new();
        
        // Parse query parameters
        let query_params = parse_query_params(request.uri().query());
        
        // Parse path segments
        let path = request.uri().path();
        let path_segments: Vec<String> = path.split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        
        Self {
            version,
            request: HyperRequest {
                inner: request,
                path_params: HashMap::new(),
                query_params,
                body_bytes: None,
            },
            response: HyperResponse {
                inner: Response::builder()
                    .status(StatusCode::OK)
                    .body(Empty::<Bytes>::new().boxed())
                    .unwrap(),
            },
            params: RwLock::new(params),
            locals: RwLock::new(locals),
            app: Some(app),
            template_manager: None,
            stream_id: None,
            role: ProtocolRole::Server,
            endpoint: None,
            path_segments,
            connection_status: ConnectionStatus::Connected,
            upgrade_context: None,
            upgrade_target: None,
        }
    }
    
    /// Create a new context for client requests
    pub fn new_client(request: Request<Body>) -> Self {
        let version = match request.version() {
            Version::HTTP_09 | Version::HTTP_10 => HttpVersion::Http1_0,
            Version::HTTP_11 => HttpVersion::Http1_1,
            Version::HTTP_2 => HttpVersion::Http2,
            Version::HTTP_3 => HttpVersion::Http3,
            _ => HttpVersion::Http1_1,
        };
        
        let query_params = parse_query_params(request.uri().query());
        
        // Parse path segments
        let path = request.uri().path();
        let path_segments: Vec<String> = path.split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        
        Self {
            version,
            request: HyperRequest {
                inner: request,
                path_params: HashMap::new(),
                query_params,
                body_bytes: None,
            },
            response: HyperResponse {
                inner: Response::builder()
                    .status(StatusCode::OK)
                    .body(Empty::<Bytes>::new().boxed())
                    .unwrap(),
            },
            params: RwLock::new(Params::new()),
            locals: RwLock::new(Locals::new()),
            app: None,
            template_manager: None,
            stream_id: None,
            role: ProtocolRole::Client,
            endpoint: None,
            path_segments,
            connection_status: ConnectionStatus::Connected,
            upgrade_context: None,
            upgrade_target: None,
        }
    }
    
    /// Set the stream ID (for HTTP/2 and HTTP/3)
    pub fn with_stream_id(mut self, stream_id: u32) -> Self {
        self.stream_id = Some(stream_id);
        self
    }
}

// ============================================================================
// HyperRequest Methods
// ============================================================================

impl HyperRequest {
    /// Get the request method
    pub fn method(&self) -> &Method {
        self.inner.method()
    }
    
    /// Get the request path
    pub fn path(&self) -> &str {
        self.inner.uri().path()
    }
    
    /// Get a query parameter
    pub fn query(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
    
    /// Get a path parameter (from URL pattern matching)
    pub fn pattern(&self, key: &str) -> Option<&String> {
        self.path_params.get(key)
    }
    
    /// Get a header value
    pub fn header(&self, name: &str) -> Option<&HeaderValue> {
        self.inner.headers().get(name)
    }
    
    /// Get all headers - Direct access to Hyper's HeaderMap
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }
    
    /// Get mutable headers - Direct access to modify headers
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        self.inner.headers_mut()
    }
    
    /// Get the HTTP version
    pub fn version(&self) -> Version {
        self.inner.version()
    }
    
    /// Get the full URI
    pub fn uri(&self) -> &hyper::Uri {
        self.inner.uri()
    }
    
    /// Get request extensions (for custom data)
    pub fn extensions(&self) -> &hyper::http::Extensions {
        self.inner.extensions()
    }
    
    /// Get mutable request extensions
    pub fn extensions_mut(&mut self) -> &mut hyper::http::Extensions {
        self.inner.extensions_mut()
    }
    
    /// Get the request body (consumes it)
    pub async fn body(self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let collected = self.inner.into_body().collect().await?;
        Ok(collected.to_bytes().to_vec())
    }
    
    /// Take the inner Hyper request (for full control)
    pub fn into_inner(self) -> Request<Body> {
        self.inner
    }
    
    /// Get a reference to the inner Hyper request
    pub fn as_inner(&self) -> &Request<Body> {
        &self.inner
    }
    
    /// Get a mutable reference to the inner Hyper request
    pub fn as_inner_mut(&mut self) -> &mut Request<Body> {
        &mut self.inner
    }
}

// ============================================================================
// HyperResponse Methods
// ============================================================================

impl HyperResponse {
    /// Get the response status
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }
    
    /// Set the response status
    pub fn set_status(&mut self, status: StatusCode) {
        *self.inner.status_mut() = status;
    }
    
    /// Get response headers - Direct access to Hyper's HeaderMap
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }
    
    /// Get mutable response headers - Direct access to modify headers
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        self.inner.headers_mut()
    }
    
    /// Add a header to the response
    pub fn add_header(&mut self, name: HeaderName, value: HeaderValue) {
        self.inner.headers_mut().insert(name, value);
    }
    
    /// Get the HTTP version
    pub fn version(&self) -> Version {
        self.inner.version()
    }
    
    /// Set the HTTP version
    pub fn set_version(&mut self, version: Version) {
        *self.inner.version_mut() = version;
    }
    
    /// Get response extensions (for custom data)
    pub fn extensions(&self) -> &hyper::http::Extensions {
        self.inner.extensions()
    }
    
    /// Get mutable response extensions
    pub fn extensions_mut(&mut self) -> &mut hyper::http::Extensions {
        self.inner.extensions_mut()
    }
    
    /// Set the response body
    pub fn set_body(&mut self, body: Vec<u8>) {
        let body = Full::new(Bytes::from(body)).boxed();
        *self.inner.body_mut() = body;
    }
    
    /// Set the response body from Bytes
    pub fn set_body_bytes(&mut self, bytes: Bytes) {
        let body = Full::new(bytes).boxed();
        *self.inner.body_mut() = body;
    }
    
    /// Set a streaming body
    pub fn set_body_stream(&mut self, body: Body) {
        *self.inner.body_mut() = body;
    }
    
    /// Set a JSON response
    pub fn json<T: serde::Serialize>(&mut self, data: T) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let json = serde_json::to_vec(&data)?;
        self.add_header(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/json")
        );
        self.set_body(json);
        Ok(())
    }
    
    /// Set a text response
    pub fn text(&mut self, text: String) {
        self.add_header(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("text/plain; charset=utf-8")
        );
        self.set_body(text.into_bytes());
    }
    
    /// Set an HTML response
    pub fn html(&mut self, html: String) {
        self.add_header(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("text/html; charset=utf-8")
        );
        self.set_body(html.into_bytes());
    }
    
    /// Take the inner Hyper response (for full control)
    pub fn into_inner(self) -> Response<Body> {
        self.inner
    }
    
    /// Get a reference to the inner Hyper response
    pub fn as_inner(&self) -> &Response<Body> {
        &self.inner
    }
    
    /// Get a mutable reference to the inner Hyper response
    pub fn as_inner_mut(&mut self) -> &mut Response<Body> {
        &mut self.inner
    }
}

// ============================================================================
// RequestContext Implementation
// ============================================================================

impl RequestContext for HyperContext {
    type Request = HyperRequest;
    type Response = HyperResponse;
    
    fn handle_error(&mut self) {
        // Set a 400 Bad Request for server, or log error for client
        match self.role {
            ProtocolRole::Server => {
                self.response.set_status(StatusCode::BAD_REQUEST);
                self.response.text("Bad Request".to_string());
            }
            ProtocolRole::Client => {
                // Log error for client
                eprintln!("Client received bad response");
            }
        }
    }
    
    fn role(&self) -> ProtocolRole {
        self.role
    }
}

impl HyperContext {
    /// Get the endpoint for parameter extraction
    pub fn endpoint(&self) -> Option<&Arc<Url<Self>>> {
        self.endpoint.as_ref()
    }
    
    /// Set the request body bytes (for form/json parsing)
    pub fn set_body_bytes(&mut self, bytes: Vec<u8>) {
        self.request.body_bytes = Some(bytes);
    }
    
    /// Signal a protocol switch to WebSocket with HTTP-specific context
    pub fn switch_to_ws(&mut self) {
        use crate::upgrade::{HttpProtocol, HttpUpgradeType, UpgradeContext, UpgradeState, UpgradeMetadata};
        
        // Determine upgrade type based on HTTP version
        let upgrade_type = match self.version {
            HttpVersion::Http2 => HttpUpgradeType::Http2ExtendedConnect {
                stream_id: self.stream_id.unwrap_or(0),
                protocol: "websocket".to_string(),
            },
            _ => HttpUpgradeType::Http1Upgrade {
                upgrade_protocol: "websocket".to_string(),
                response_sent: false,
            },
        };
        
        // Create upgrade context
        let upgrade_ctx = UpgradeContext {
            target_protocol: HttpProtocol::WebSocket,
            upgrade_type,
            state: UpgradeState::Requested,
            metadata: UpgradeMetadata::default(),
            initiated_at: std::time::Instant::now(),
        };
        
        // Store upgrade context
        self.upgrade_context = Some(upgrade_ctx);
        
        // Set upgrade target
        self.upgrade_target = Some(HttpProtocol::WebSocket);
        
        // For now, keep using TypeId for ConnectionStatus to maintain compatibility with hotaru_core
        // We can later update hotaru_core if needed
        use crate::websocket::WebSocketProtocol;
        self.connection_status = ConnectionStatus::SwitchProtocol(
            std::any::TypeId::of::<WebSocketProtocol>()
        );
    }
    
    /// Signal a protocol switch to HTTP/2 with upgrade context
    pub fn switch_to_h2(&mut self) {
        use crate::upgrade::{HttpProtocol, HttpUpgradeType, UpgradeContext, UpgradeState, UpgradeMetadata};
        
        // Create h2c upgrade context
        let upgrade_ctx = UpgradeContext {
            target_protocol: HttpProtocol::Http2,
            upgrade_type: HttpUpgradeType::Http2Cleartext {
                http2_settings: None, // Would extract from HTTP2-Settings header
                direct: false,
            },
            state: UpgradeState::Requested,
            metadata: UpgradeMetadata::default(),
            initiated_at: std::time::Instant::now(),
        };
        
        // Store upgrade context
        self.upgrade_context = Some(upgrade_ctx);
        
        // Set upgrade target
        self.upgrade_target = Some(HttpProtocol::Http2);
        
        // For now, keep using TypeId for ConnectionStatus to maintain compatibility with hotaru_core
        use crate::protocol::HyperHttp2;
        self.connection_status = ConnectionStatus::SwitchProtocol(
            std::any::TypeId::of::<HyperHttp2>()
        );
    }
    
    /// Signal a generic protocol switch
    pub fn switch_protocol(&mut self, protocol_type_id: std::any::TypeId) {
        self.connection_status = ConnectionStatus::SwitchProtocol(protocol_type_id);
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_query_params(query: Option<&str>) -> HashMap<String, String> {
    let mut params = HashMap::new();
    
    if let Some(query) = query {
        for pair in query.split('&') {
            if let Some(eq_pos) = pair.find('=') {
                let key = &pair[..eq_pos];
                let value = &pair[eq_pos + 1..];
                params.insert(
                    urlencoding::decode(key).unwrap_or_else(|_| key.into()).to_string(),
                    urlencoding::decode(value).unwrap_or_else(|_| value.into()).to_string(),
                );
            } else {
                params.insert(
                    urlencoding::decode(pair).unwrap_or_else(|_| pair.into()).to_string(),
                    String::new(),
                );
            }
        }
    }
    
    params
}

// ============================================================================
// Compatibility Methods for Hotaru API
// ============================================================================

impl HyperContext {
    /// Render a template (using akari)
    pub async fn render(&self, template_name: &str, data: HashMap<String, Value>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(tm) = &self.template_manager {
            Ok(tm.render(template_name, &data)?)
        } else {
            Err("Template manager not configured".into())
        }
    }
    
    /// Clone params for middleware use
    pub fn params_clone(&self) -> Params {
        let params = self.params.read().unwrap();
        // Create a new Params and copy the data
        let mut new_params = Params::new();
        // Note: This assumes Params has some way to iterate/copy
        // We'll just return a new empty one for now
        new_params
    }
    
    /// Clone locals for middleware use
    pub fn locals_clone(&self) -> Locals {
        let locals = self.locals.read().unwrap();
        // Create a new Locals and copy the data
        let mut new_locals = Locals::new();
        // Note: This assumes Locals has some way to iterate/copy
        // We'll just return a new empty one for now
        new_locals
    }
    
    /// Get as Any for downcasting
    pub fn as_any(&self) -> &dyn Any {
        self
    }
    
    /// Get the request
    pub fn request(&self) -> &HyperRequest {
        &self.request
    }
    
    /// Get the response
    pub fn response(&self) -> &HyperResponse {
        &self.response
    }
    
    /// Get mutable response
    pub fn response_mut(&mut self) -> &mut HyperResponse {
        &mut self.response
    }
    
    // ========================================================================
    // Methods for compatibility with HttpContext
    // ========================================================================
    
    /// Get the request method (compatibility with HttpContext)
    pub fn method(&self) -> &Method {
        self.request.method()
    }
    
    /// Get the request path (compatibility with HttpContext)
    pub fn path(&self) -> &str {
        self.request.path()
    }
    
    /// Get a path parameter from URL pattern matching
    pub fn pattern<A: AsRef<str>>(&self, name: A) -> Option<String> {
        // First check if we have it in path_params (for backward compatibility)
        if let Some(value) = self.request.path_params.get(name.as_ref()) {
            return Some(value.clone());
        }
        
        // Otherwise, try to extract from endpoint and path segments
        if let Some(endpoint) = &self.endpoint {
            if let Some(index) = endpoint.match_seg_name_with_index(name) {
                return self.path_segments.get(index).cloned();
            }
        }
        
        None
    }
    
    /// Get the application reference
    pub fn app(&self) -> Option<Arc<App>> {
        self.app.clone()
    }
    
    /// Parse and get form data using serde_urlencoded
    pub async fn form(&mut self) -> Option<UrlEncodedForm> {
        // Check if this is form-encoded data
        let content_type = self.request.header("content-type")?;
        let content_type_str = content_type.to_str().ok()?;
        
        if !content_type_str.contains("application/x-www-form-urlencoded") {
            return None;
        }
        
        // Parse form data from stored body bytes using serde_urlencoded
        let body_bytes = self.request.body_bytes.as_ref()?;
        
        // Use serde_urlencoded for proper form parsing
        match serde_urlencoded::from_bytes::<HashMap<String, String>>(body_bytes) {
            Ok(data) => Some(UrlEncodedForm { data }),
            Err(_) => None,
        }
    }
    
    /// Parse form data into a specific type
    pub async fn form_typed<T: DeserializeOwned>(&mut self) -> Option<T> {
        // Check if this is form-encoded data
        let content_type = self.request.header("content-type")?;
        let content_type_str = content_type.to_str().ok()?;
        
        if !content_type_str.contains("application/x-www-form-urlencoded") {
            return None;
        }
        
        // Parse directly into the requested type
        let body_bytes = self.request.body_bytes.as_ref()?;
        serde_urlencoded::from_bytes(body_bytes).ok()
    }
    
    /// Parse and get JSON data
    pub async fn json<T: serde::de::DeserializeOwned>(&mut self) -> Option<T> {
        // Check if this is JSON data
        if let Some(content_type) = self.request.header("content-type") {
            let content_type_str = content_type.to_str().ok()?;
            if !content_type_str.contains("application/json") {
                return None;
            }
        }
        
        // Parse JSON from stored body bytes
        let body_bytes = self.request.body_bytes.as_ref()?;
        serde_json::from_slice(body_bytes).ok()
    }
    
    /// Get cookies (placeholder - needs implementation)
    pub fn get_cookies(&self) -> HashMap<String, String> {
        // TODO: Parse cookies from headers
        HashMap::new()
    }
    
    /// Get a specific cookie
    pub fn get_cookie(&self, key: &str) -> Option<String> {
        self.get_cookies().get(key).cloned()
    }
    
}

// ============================================================================
// Response Helper Functions
// ============================================================================

/// Create a JSON response
pub fn json_response<T: serde::Serialize>(data: T) -> HyperContext {
    let mut ctx = HyperContext::new_client(
        Request::builder()
            .method("GET")
            .uri("/")
            .body(Empty::<Bytes>::new().boxed())
            .unwrap()
    );
    let _ = ctx.response.json(data);
    ctx
}

/// Create a text response  
pub fn text_response(text: impl Into<String>) -> HyperContext {
    let mut ctx = HyperContext::new_client(
        Request::builder()
            .method("GET")
            .uri("/")
            .body(Empty::<Bytes>::new().boxed())
            .unwrap()
    );
    ctx.response.text(text.into());
    ctx
}

/// Create an HTML response
pub fn html_response(html: impl Into<String>) -> HyperContext {
    let mut ctx = HyperContext::new_client(
        Request::builder()
            .method("GET")
            .uri("/")
            .body(Empty::<Bytes>::new().boxed())
            .unwrap()
    );
    ctx.response.html(html.into());
    ctx
}

/// Create a response that signals a protocol switch
pub fn switch_protocol_response(response: Response<Body>, target_protocol: std::any::TypeId) -> HyperContext {
    let mut ctx = HyperContext::new_client(
        Request::builder()
            .method("GET")
            .uri("/")
            .body(Empty::<Bytes>::new().boxed())
            .unwrap()
    );
    ctx.response.inner = response;
    ctx.connection_status = ConnectionStatus::SwitchProtocol(target_protocol);
    ctx
}