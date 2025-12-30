use crate::app::application::App;
use crate::connection::error::ConnectionError;
use crate::connection::{
    ConnectionBuilder, ConnectionStatus, ProtocolRole, RequestContext, TcpConnectionStream,
};
use std::net::{IpAddr, SocketAddr};
use crate::debug_log;
use crate::extensions::{Locals, Params};
use crate::http::cookie::{Cookie, CookieMap};
use crate::http::request::HttpRequest;
use crate::http::safety::HttpSafety;
use crate::http::{
    body::HttpBody,
    form::{MultiForm, UrlEncodedForm},
    http_value::HttpMethod,
    meta::HttpMeta,
    response::HttpResponse,
};
use crate::url::Url;
use akari::Value; 
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{BufReader, BufWriter, ReadHalf, WriteHalf};

use super::http_value::StatusCode;
use super::response::response_templates;

/// Executable context - determines what's available for execution
pub enum Executable {
    /// Server context with App and URL endpoint
    Request {
        app: Arc<App>,
        endpoint: Arc<Url<HttpContext>>,
    },
    /// Client context (empty for now, will be extended later)
    Response,
}

/// Unified HTTP context for both server and client operations.
///
/// This context flows through handlers and middleware, supporting both
/// server-side request handling and client-side response processing.
pub struct HttpContext {
    pub request: HttpRequest,
    pub response: HttpResponse,

    // Execution context determines available operations
    pub executable: Executable,

    // Additional fields
    pub host: Option<String>, // Used by client for target host
    pub safety: HttpSafety,

    // Socket addresses
    remote_addr: Option<SocketAddr>,
    local_addr: Option<SocketAddr>,

    // Shared fields for middleware/handlers
    pub params: Params,
    pub locals: Locals,
}

// Type alias for backward compatibility
pub type HttpReqCtx = HttpContext;

/// Placeholder address for uninitialized or unknown connections.
/// `0.0.0.0:0` indicates that no socket address information is available.
const UNSET_ADDR: SocketAddr = SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
    0,
);

impl HttpContext {
    /// Creates a new server context with socket addresses.
    pub fn new_server(
        app: Arc<App>,
        endpoint: Arc<Url<HttpContext>>,
        request: HttpRequest,
        remote_addr: Option<SocketAddr>,
        local_addr: Option<SocketAddr>,
    ) -> Self {
        Self {
            request,
            response: HttpResponse::default(),
            executable: Executable::Request { app, endpoint },
            host: None,
            safety: HttpSafety::default(),
            remote_addr,
            local_addr,
            params: Default::default(),
            locals: Default::default(),
        }
    }

    /// Creates a new client context
    pub fn new_client(host: String, safety: HttpSafety) -> Self {
        Self {
            request: HttpRequest::default(),
            response: HttpResponse::default(),
            executable: Executable::Response,
            host: Some(host),
            safety,
            remote_addr: None,
            local_addr: None,
            params: Default::default(),
            locals: Default::default(),
        }
    }

    // /// Creates a new Request Context (backward compatibility)
    // #[deprecated(note = "Use new_server() with socket addresses instead")]
    // pub fn new(app: Arc<App>, endpoint: Arc<Url<HttpContext>>, request: HttpRequest) -> Self {
    //     Self::new_server(app, endpoint, request, UNSET_ADDR, UNSET_ADDR)
    // }

    // /// Handles the request by parsing it and creating a new `HttpContext`.
    // #[deprecated(note = "Use new_server() with socket addresses instead")]
    // pub async fn handle(
    //     app: Arc<App>,
    //     root_handler: Arc<Url<HttpContext>>,
    //     request: HttpRequest,
    // ) -> Self {
    //     let endpoint = root_handler.walk_str(&request.meta.path()).await;
    //     Self::new(app.clone(), endpoint.clone(), request)
    // }

    // =========================================================================
    // Socket Address Accessors
    // =========================================================================

    /// Returns the client's socket address (IP and port).
    /// For server context, this is the remote peer that connected.
    /// For client context, this is the server we connected to.
    /// If unknown, this returns `None`.
    #[inline]
    pub fn client_ip(&self) -> Option<SocketAddr> {
        self.remote_addr
    }

    /// Returns the client's socket address (IP and port), or `0.0.0.0:0` if unknown.
    #[inline]
    pub fn client_ip_or_default(&self) -> SocketAddr {
        self.remote_addr.unwrap_or(UNSET_ADDR)
    }

    /// Returns just the client's IP address without the port.
    /// If unknown, this returns `None`.
    #[inline]
    pub fn client_ip_only(&self) -> Option<IpAddr> {
        self.remote_addr.map(|addr| addr.ip())
    }

    /// Returns just the client's IP address without the port, or `0.0.0.0` if unknown.
    #[inline]
    pub fn client_ip_only_or_default(&self) -> IpAddr {
        self.remote_addr.map(|addr| addr.ip()).unwrap_or(UNSET_ADDR.ip())
    }

    /// Returns the server's bound socket address.
    /// For server context, this is the local address we're listening on.
    /// For client context, this is our local ephemeral port.
    /// If unknown, this returns `None`.
    #[inline]
    pub fn server_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    /// Returns the server's bound socket address, or `0.0.0.0:0` if unknown.
    #[inline]
    pub fn server_addr_or_default(&self) -> SocketAddr {
        self.local_addr.unwrap_or(UNSET_ADDR)
    }

    /// Returns the remote socket address (alias for client_ip in server context).
    /// If unknown, this returns `None`.
    #[inline]
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }

    /// Returns the remote socket address, or `0.0.0.0:0` if unknown.
    #[inline]
    pub fn remote_addr_or_default(&self) -> SocketAddr {
        self.remote_addr.unwrap_or(UNSET_ADDR)
    }

    /// Returns the local socket address (alias for server_addr).
    /// If unknown, this returns `None`.
    #[inline]
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    /// Returns the local socket address, or `0.0.0.0:0` if unknown.
    #[inline]
    pub fn local_addr_or_default(&self) -> SocketAddr {
        self.local_addr.unwrap_or(UNSET_ADDR)
    }

    pub async fn read_request(
        app: Arc<App>,
        reader: &mut BufReader<ReadHalf<TcpConnectionStream>>,
    ) -> Result<HttpRequest, ConnectionError> {
        Ok(HttpRequest::parse_lazy(
            reader,
            app.config.get::<HttpSafety>().unwrap_or_default(),
            app.get_mode() == crate::app::application::RunMode::Build,
        )
        .await)
    }

    /// Sends the response
    pub async fn send_response(
        response: HttpResponse,
        writer: &mut BufWriter<WriteHalf<TcpConnectionStream>>,
    ) {
        let _ = response.send(writer).await;
    }

    /// Runs the endpoint and sending the response.
    ///
    /// # Return
    ///
    /// Returns the response and a boolean indicating whether the connection should be closed.
    /// Response is the response of the endpoint, and the boolean indicates whether the connection should be closed.
    pub async fn run(mut self) -> Result<(HttpResponse, ConnectionStatus), ConnectionError> {
        if let Some(endpoint) = self.endpoint() {
            debug_log!("HTTP Context: Found endpoint, checking request");
            if let Err(s) = self.request_check(&endpoint) {
                debug_log!("HTTP Context: Request check failed with status: {:?}", s);
                return Ok((
                    response_templates::return_status(s),
                    ConnectionStatus::Stopped,
                ));
            };
            debug_log!("HTTP Context: Running endpoint handler");
            let result = endpoint.run(self).await;
            debug_log!("HTTP Context: Handler completed");
            Ok((result.response, ConnectionStatus::Stopped))
        } else {
            debug_log!("HTTP Context: No endpoint available (client context)");
            // No endpoint available (client context)
            Ok((self.response, ConnectionStatus::Stopped))
        }
    }

    /// Checks whether the request fulfills the endpoint's security requirements.
    pub fn request_check(&mut self, endpoint: &Arc<Url<HttpContext>>) -> Result<(), StatusCode> {
        let config = endpoint.get_params::<HttpSafety>().unwrap_or_default();
        // println!(
        //     "Checking request: {:?} {}{} ",config,self.request.meta.method(),config.check_method(&self.request.meta.method())
        // );
        if !config.check_body_size(self.request.meta.get_content_length().unwrap_or(0)) {
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }
        if !config.check_method(&self.request.meta.method()) {
            return Err(StatusCode::METHOD_NOT_ALLOWED);
        }
        if !config.check_content_type(&self.request.meta.get_content_type().unwrap_or_default()) {
            return Err(StatusCode::UNSUPPORTED_MEDIA_TYPE);
        }
        return Ok(());
    }

    /// Returns the meta in the request as reference
    pub fn meta(&mut self) -> &mut HttpMeta {
        &mut self.request.meta
    }

    /// Returns the Arc<App> if this is a server context
    pub fn app(&self) -> Option<Arc<App>> {
        match &self.executable {
            Executable::Request { app, .. } => Some(app.clone()),
            Executable::Response => None,
        }
    }

    /// Returns the endpoint URL if this is a server context
    pub fn endpoint(&self) -> Option<Arc<Url<HttpContext>>> {
        match &self.executable {
            Executable::Request { endpoint, .. } => Some(endpoint.clone()),
            Executable::Response => None,
        }
    }

    /// Parses the body of the request, reading it into the `HttpBody` field of the request.
    /// Note that request body will not be automatically parsed unless this function is called
    /// The automatic parsing is not recommended, as it can lead to performance issues and security vulnerabilities.
    /// If you didn't parse body, the body will be `HttpBody::Unparsed`.
    pub async fn parse_body(&mut self) {
        let safety_settings = if let Some(endpoint) = self.endpoint() {
            let mut settings = endpoint.get_params::<HttpSafety>().unwrap_or_default();
            settings.update(&endpoint.get_params::<HttpSafety>().unwrap_or_default());
            settings
        } else {
            self.safety.clone()
        };

        // Take the body out, replacing it with the default temporarily
        let body = std::mem::take(&mut self.request.body);
        self.request.body = body.parse_buffer(&safety_settings);
    }

    /// Returns the body of the request as a reference to `HttpBody`.
    pub async fn form(&mut self) -> Option<&UrlEncodedForm> {
        self.parse_body().await; // Await the Future<Output = ()>
        if let HttpBody::Form(ref data) = self.request.body {
            Some(data)
        } else {
            None
        }
    }

    /// Returns the body of the request as a reference to `UrlEncodedForm`, or an empty form if not present.
    pub async fn form_or_default(&mut self) -> &UrlEncodedForm {
        match self.form().await {
            Some(form) => form,
            None => {
                static EMPTY: Lazy<UrlEncodedForm> = Lazy::new(|| HashMap::new().into());
                &EMPTY
            }
        }
    }

    /// Returns the body of the request as a reference to `MultiForm`.
    pub async fn files(&mut self) -> Option<&MultiForm> {
        self.parse_body().await; // Await the Future<Output = ()>
        if let HttpBody::Files(ref data) = self.request.body {
            Some(data)
        } else {
            None
        }
    }

    /// Returns the body of the request as a reference to `MultiForm`, or an empty form if not present.
    pub async fn files_or_default(&mut self) -> &MultiForm {
        match self.files().await {
            Some(files) => files,
            None => {
                static EMPTY: Lazy<MultiForm> = Lazy::new(|| HashMap::new().into());
                &EMPTY
            }
        }
    }

    /// Returns the body of the request as a reference to `HttpBody::Binary`.
    pub async fn json(&mut self) -> Option<&Value> {
        self.parse_body().await; // Await the Future<Output = ()>
        if let HttpBody::Json(ref data) = self.request.body {
            Some(data)
        } else {
            None
        }
    }

    /// Returns the body of the request as a reference to `HttpBody::Binary`, or an empty JSON if not present.
    pub async fn json_or_default(&mut self) -> &Value {
        match self.json().await {
            Some(json) => json,
            None => {
                static EMPTY: Lazy<Value> = Lazy::new(|| Value::new(""));
                &EMPTY
            }
        }
    }

    /// Get a path segment by index position
    /// For example, in "/api/users/123", segment(0) = "api", segment(1) = "users", segment(2) = "123"
    pub fn segment(&mut self, index: usize) -> String {
        self.request.meta.get_path(index)
    }

    /// Get the whole path
    pub fn path(&self) -> String {
        self.request.meta.path()
    }

    /// Get a named path parameter from the URL pattern
    /// For example, with pattern "/users/<id>", param("id") returns the value in place of <id>
    pub fn param<A: AsRef<str>>(&mut self, name: A) -> Option<String> {
        self.endpoint().and_then(|endpoint| {
            endpoint
                .match_seg_name_with_index(name)
                .map(|index| self.request.meta.get_path(index))
        })
    }

    /// Alias for param() - kept for backward compatibility
    pub fn pattern<A: AsRef<str>>(&mut self, name: A) -> Option<String> {
        self.param(name)
    }

    /// Get a query parameter value by key
    /// For example, in "/search?q=rust&limit=10", query("q") returns Some("rust")
    pub fn query<T: Into<String>>(&mut self, key: T) -> Option<String> {
        self.request.meta.get_url_args(key)
    }

    /// Get the preferred by the user
    pub fn get_preferred_language(&mut self) -> Option<String> {
        self.request
            .meta
            .get_lang()
            .map(|lang_dict| lang_dict.most_preferred())
    }

    /// Get the preferred by the user with a default value
    pub fn get_preferred_language_or_default<T: AsRef<str>>(&mut self, default: T) -> String {
        self.get_preferred_language()
            .unwrap_or_else(|| default.as_ref().to_string())
    }

    /// Returns the method of the request.
    pub fn method(&mut self) -> HttpMethod {
        self.request.meta.method()
    }

    /// Convenience method to get request headers directly.
    /// Avoids the long chain: req.request.meta.header
    pub fn headers(&self) -> &HashMap<String, super::meta::HeaderValue> {
        &self.request.meta.header
    }

    /// Convenience method to get a specific header value.
    pub fn header(&self, key: &str) -> Option<&super::meta::HeaderValue> {
        self.request.meta.header.get(key)
    }

    /// Convenience method to get a header value as a string.
    /// Returns the first value if multiple values exist.
    pub fn header_str(&self, key: &str) -> Option<&str> {
        self.request.meta.header.get(key).and_then(|hv| match hv {
            super::meta::HeaderValue::Single(s) => Some(s.as_str()),
            super::meta::HeaderValue::Multiple(v) => v.first().map(|s| s.as_str()),
        })
    }

    /// Convenience method to check if a header exists.
    pub fn has_header(&self, key: &str) -> bool {
        self.request.meta.header.contains_key(key)
    }

    /// Get teh full cookie map
    pub fn get_cookies(&mut self) -> &CookieMap {
        self.request.meta.get_cookies()
    }

    /// Get a single cookie
    pub fn get_cookie(&mut self, key: &str) -> Option<Cookie> {
        self.request.meta.get_cookie(key)
    }

    /// Get a cookie. If not found a default cookie will be returned
    pub fn get_cookie_or_default<T: AsRef<str>>(&mut self, key: T) -> Cookie {
        self.request.meta.get_cookie_or_default(key)
    }

    // ========================================================================
    // Response convenience methods
    // ========================================================================

    /// Get a mutable reference to the response for chaining.
    pub fn response_mut(&mut self) -> &mut HttpResponse {
        &mut self.response
    }

    /// Set the response status code.
    pub fn set_status(&mut self, code: u16) -> &mut Self {
        self.response.meta.start_line.set_status_code(code);
        self
    }

    /// Add a response header.
    pub fn add_response_header(&mut self, key: String, value: String) -> &mut Self {
        self.response.meta.set_attribute(key, value);
        self
    }

    /// Set the response body.
    pub fn set_body(&mut self, body: HttpBody) -> &mut Self {
        self.response.body = body;
        self
    }
}

impl RequestContext for HttpContext {
    type Request = HttpRequest;
    type Response = HttpResponse;

    fn handle_error(&mut self) {
        match &self.executable {
            Executable::Request { .. } => {
                // Server: return 404
                self.response = response_templates::html_response(
                    "<h1>404 Not Found</h1><br><p>This route is not found</p>",
                )
                .status(404);
            }
            Executable::Response => {
                // Client: set default error response
                self.response = HttpResponse::default();
            }
        }
    }

    fn role(&self) -> ProtocolRole {
        match &self.executable {
            Executable::Request { .. } => ProtocolRole::Server,
            Executable::Response => ProtocolRole::Client,
        }
    }
}

impl HttpContext {
    pub fn bad_request(&mut self) {
        self.handle_error();
    }
}

// Type alias for backward compatibility with client code
pub type HttpResCtx = HttpContext;

impl HttpContext {
    /// Creates a client context for sending requests (backward compatibility)
    pub fn new_res(config: HttpSafety, host: impl Into<String>) -> Self {
        Self::new_client(host.into(), config)
    }

    /// Sends a request to the given host and returns a `HttpResCtx` context.
    /// This function will automatically determine whether to use HTTP or HTTPS based on the host string.
    pub async fn send_request<T: Into<String>>(
        host: T,
        mut request: HttpRequest,
        safety_config: HttpSafety,
    ) -> Result<HttpResponse, ConnectionError> {
        // Test whether the host uses https
        let host_str = host.into();
        let (is_https, without_scheme) = if host_str.starts_with("https://") {
            (true, host_str.trim_start_matches("https://"))
        } else if host_str.starts_with("http://") {
            (false, host_str.trim_start_matches("http://"))
        } else {
            (false, host_str.as_str())
        };

        // Find last colon with trailing digits
        let mut host_part = without_scheme;
        let mut port = if is_https { 443 } else { 80 };
        let mut explicit_port = false;

        if let Some(colon_pos) = without_scheme.rfind(':') {
            let port_part = &without_scheme[colon_pos + 1..];

            // Check if port part is numeric (1-5 digits)
            if !port_part.is_empty()
                && port_part.len() <= 5
                && port_part.chars().all(|c| c.is_ascii_digit())
            {
                if let Ok(parsed_port) = port_part.parse::<u16>() {
                    port = parsed_port;
                    host_part = &without_scheme[..colon_pos];
                    explicit_port = true;
                }
            }
        }

        // Auto set host if host is not set
        match request.meta.get_host() {
            None => {
                // Host is NOT set, so set it
                if explicit_port {
                    request
                        .meta
                        .set_host(Some(format!("{}:{}", host_part, port)));
                } else {
                    request.meta.set_host(Some(host_part.to_string()));
                }
            }
            Some(_) => {} // Host is already set, do nothing
        }

        if let Some(colon_pos) = without_scheme.rfind(':') {
            let port_part = &without_scheme[colon_pos + 1..];

            // Check if port part is numeric (1-5 digits)
            if !port_part.is_empty()
                && port_part.len() <= 5
                && port_part.chars().all(|c| c.is_ascii_digit())
            {
                if let Ok(parsed_port) = port_part.parse::<u16>() {
                    port = parsed_port;
                    host_part = &without_scheme[..colon_pos];
                }
            }
        }

        let (read, write) = ConnectionBuilder::new(host_part, port)
            .protocol(crate::connection::builder::Protocol::HTTP)
            .tls(is_https)
            .connect()
            .await?
            .split();

        let mut reader = BufReader::new(read);
        let mut writer = BufWriter::new(write);

        // Write the HTTP request frame
        HttpContext::write_frame(&mut writer, request).await?;
        // Read the HTTP response frame
        let response = HttpContext::read_next_frame(&safety_config, &mut reader).await?;
        Ok(response)
    }

    pub fn request(&mut self, mut request: HttpRequest) {
        if request.meta.get_host().is_none() {
            if let Some(ref host) = self.host {
                request.meta.set_host(Some(host.clone()));
            }
        };
        self.request = request;
    }

    /// Write an HTTP request frame to the stream
    pub async fn write_frame(
        write_stream: &mut BufWriter<WriteHalf<TcpConnectionStream>>,
        request_frame: HttpRequest,
    ) -> Result<(), ConnectionError> {
        request_frame
            .send(write_stream)
            .await
            .map_err(|e| ConnectionError::IoError(e))?;
        Ok(())
    }

    /// Read an HTTP response frame from the stream
    pub async fn read_next_frame(
        config: &HttpSafety,
        read_stream: &mut BufReader<ReadHalf<TcpConnectionStream>>,
    ) -> Result<HttpResponse, ConnectionError> {
        Ok(HttpResponse::parse_lazy(read_stream, config, false).await)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        http::{
            context::HttpResCtx,
            request::request_templates::get_request,
            safety::HttpSafety,
        },
    };
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    // =========================================================================
    // Socket Address Accessor Tests
    // =========================================================================

    #[test]
    fn test_client_ip_with_address() {
        use crate::app::application::App;
        use crate::http::request::HttpRequest;
        use crate::url::Url;
        use std::sync::Arc;

        let app = App::new().build();
        let endpoint = Arc::new(Url::<super::HttpContext>::default());
        let request = HttpRequest::default();
        let remote = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 54321);
        let local = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);

        let ctx = super::HttpContext::new_server(
            app,
            endpoint,
            request,
            Some(remote),
            Some(local),
        );

        // Test client_ip()
        assert_eq!(ctx.client_ip(), Some(remote));
        assert_eq!(ctx.client_ip_or_default(), remote);

        // Test client_ip_only()
        assert_eq!(ctx.client_ip_only(), Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))));
        assert_eq!(ctx.client_ip_only_or_default(), IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)));

        // Test server_addr()
        assert_eq!(ctx.server_addr(), Some(local));
        assert_eq!(ctx.server_addr_or_default(), local);

        // Test aliases
        assert_eq!(ctx.remote_addr(), Some(remote));
        assert_eq!(ctx.remote_addr_or_default(), remote);
        assert_eq!(ctx.local_addr(), Some(local));
        assert_eq!(ctx.local_addr_or_default(), local);
    }

    #[test]
    fn test_client_ip_without_address() {
        use crate::app::application::App;
        use crate::http::request::HttpRequest;
        use crate::url::Url;
        use std::sync::Arc;

        let app = App::new().build();
        let endpoint = Arc::new(Url::<super::HttpContext>::default());
        let request = HttpRequest::default();

        let ctx = super::HttpContext::new_server(
            app,
            endpoint,
            request,
            None,  // No remote address
            None,  // No local address
        );

        let unset = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

        // Test client_ip() returns None
        assert_eq!(ctx.client_ip(), None);
        assert_eq!(ctx.client_ip_or_default(), unset);

        // Test client_ip_only() returns None
        assert_eq!(ctx.client_ip_only(), None);
        assert_eq!(ctx.client_ip_only_or_default(), IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));

        // Test server_addr() returns None
        assert_eq!(ctx.server_addr(), None);
        assert_eq!(ctx.server_addr_or_default(), unset);

        // Test aliases
        assert_eq!(ctx.remote_addr(), None);
        assert_eq!(ctx.remote_addr_or_default(), unset);
        assert_eq!(ctx.local_addr(), None);
        assert_eq!(ctx.local_addr_or_default(), unset);
    }

    #[test]
    fn test_client_context_has_no_addresses() {
        let ctx = super::HttpContext::new_client(
            "example.com".to_string(),
            HttpSafety::default(),
        );

        // Client contexts start with no addresses
        assert_eq!(ctx.client_ip(), None);
        assert_eq!(ctx.server_addr(), None);
    }

    #[test]
    fn test_ipv6_address() {
        use crate::app::application::App;
        use crate::http::request::HttpRequest;
        use crate::url::Url;
        use std::net::Ipv6Addr;
        use std::sync::Arc;

        let app = App::new().build();
        let endpoint = Arc::new(Url::<super::HttpContext>::default());
        let request = HttpRequest::default();
        let remote = SocketAddr::new(
            IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)),
            54321
        );
        let local = SocketAddr::new(
            IpAddr::V6(Ipv6Addr::LOCALHOST),
            8080
        );

        let ctx = super::HttpContext::new_server(
            app,
            endpoint,
            request,
            Some(remote),
            Some(local),
        );

        assert_eq!(ctx.client_ip(), Some(remote));
        assert_eq!(ctx.client_ip_only(), Some(IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1))));
        assert_eq!(ctx.server_addr(), Some(local));
    }

    // =========================================================================
    // HTTP Client Tests
    // =========================================================================

    // #[tokio::test]
    // async fn request_a_page() {
    //     let builder = ConnectionBuilder::new("example.com", 443)
    //         .protocol(Protocol::HTTP)
    //         .tls(true);
    //     let connection = builder.connect().await.unwrap();
    //     let mut request = HttpResCtx::new(
    //         connection,
    //         HttpSafety::new().with_max_body_size(25565),
    //         "example.com",
    //     );
    //     let _ = request.process(request_templates::get_request("/")).await;
    //     request.parse_response().await;
    //     // println!("{:?}, {:?}", request.response.meta, request.response.body);
    // }

    #[tokio::test]
    async fn request_another_page() {
        let response = HttpResCtx::send_request(
            "https://api.pmine.org",
            get_request("/num/change/lhsduifhsjdbczfjgszjdhfgxyjey/36/2"),
            HttpSafety::new().with_max_body_size(25565),
        )
        .await
        .unwrap();
        println!("{:?}, {:?}", response.meta, response.body);
    }

    #[tokio::test]
    async fn request_chunked_page() {
        let response = HttpResCtx::send_request(
            "https://api.pmine.org",
            get_request("/num/c2"),
            HttpSafety::new().with_max_body_size(25565),
        )
        .await
        .unwrap();
        println!("{:?}, {:?}", response.meta, response.body);
    }

    /// Test requires a local server running on port 3003
    /// Run with: cargo test --lib -- --ignored localhost
    #[tokio::test]
    #[ignore = "requires local server on port 3003"]
    async fn localhost() {
        let response = HttpResCtx::send_request(
            "http://localhost:3003",
            get_request("/"),
            HttpSafety::new().with_max_body_size(25565),
        )
        .await
        .unwrap();
        println!("{:?}, {:?}", response.meta, response.body);
    }
}
