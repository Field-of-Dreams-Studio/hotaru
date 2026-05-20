use hotaru_core::app::common::{RunMode, RuntimeConfig};
use hotaru_core::connection::error::ConnectionError;
use hotaru_core::connection::{ConnStream, Outbound, TransportSpec};
use hotaru_core::debug_log;
use hotaru_core::extensions::{Locals, Params};
use hotaru_core::url::UrlNode;
use akari::Value;
use hotaru_core::protocol::{BoxProtocolError, ProtocolError, ProtocolRole, RequestContext};

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::io::{AsyncBufRead, AsyncWrite, BufReader, BufWriter};

use crate::message::body::HttpBody;
use crate::message::http_value::{HttpMethod, StatusCode};
use crate::protocol::HttpError;
use crate::message::meta::HttpMeta;
use crate::message::request::HttpRequest;
use crate::message::response::{HttpResponse, response_templates};
use crate::security::safety::HttpSafety;

use crate::util::cookie::{Cookie, CookieMap};
use crate::util::form::{MultiForm, UrlEncodedForm};

/// Executable context - determines what's available for execution
pub enum Executable<TS: TransportSpec = crate::connection::tcp::TcpTransport> {
    /// Server context with runtime config and matched endpoint.
    Request {
        runtime: Arc<RuntimeConfig>,
        endpoint: Arc<UrlNode<HttpContext<TS>, TS>>,
    },
    /// Client context (empty for now, will be extended later)
    Response,
}

/// Unified HTTP context for both server and client operations.
///
/// This context flows through handlers and middleware, supporting both
/// server-side request handling and client-side response processing.
pub struct HttpContext<TS: TransportSpec = crate::connection::tcp::TcpTransport> {
    pub request: HttpRequest,
    pub response: HttpResponse,

    // Execution context determines available operations
    pub executable: Executable<TS>,

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
pub type HttpReqCtx<TS = crate::connection::tcp::TcpTransport> = HttpContext<TS>;

/// Placeholder address for uninitialized or unknown connections.
/// `0.0.0.0:0` indicates that no socket address information is available.
const UNSET_ADDR: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)), 0);

impl<TS: TransportSpec> HttpContext<TS> {
    /// Creates a new server context with socket addresses.
    pub fn new_server(
        runtime: Arc<RuntimeConfig>,
        endpoint: Arc<UrlNode<HttpContext<TS>, TS>>,
        request: HttpRequest,
        remote_addr: Option<SocketAddr>,
        local_addr: Option<SocketAddr>,
    ) -> Self {
        Self {
            request,
            response: HttpResponse::default(),
            executable: Executable::Request { runtime, endpoint },
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
            executable: Executable::<TS>::Response,
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
        self.remote_addr
            .map(|addr| addr.ip())
            .unwrap_or(UNSET_ADDR.ip())
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

    pub async fn read_request<R>(
        runtime: Arc<RuntimeConfig>,
        reader: &mut R,
    ) -> Result<HttpRequest, ConnectionError>
    where
        R: AsyncBufRead + Unpin,
    {
        Ok(HttpRequest::parse_lazy(
            reader,
            &runtime.get_config::<HttpSafety>().unwrap_or_default(),
            runtime.mode() == RunMode::Build,
        )
        .await)
    }

    /// Sends the response
    pub async fn send_response<W>(response: HttpResponse, writer: &mut W)
    where
        W: AsyncWrite + Unpin,
    {
        let _ = response.send(writer).await;
    }

    /// Runs the endpoint and returns the context with the response set.
    ///
    /// # Return
    ///
    /// Returns `Ok(HttpContext)` with the response populated, or `Err(BoxProtocolError)`
    /// if the endpoint chain fails. Keep-alive is managed by `Http1Channel.open`.
    pub async fn run(mut self) -> Result<HttpContext<TS>, BoxProtocolError> {
        if let Some(endpoint) = self.endpoint() {
            debug_log!("HTTP Context: Found endpoint, checking request");
            if let Err(err) = self.request_check(&endpoint) {
                debug_log!("HTTP Context: Request check failed: {:?}", err);
                let status: StatusCode = (&err).into();
                self.response = response_templates::return_status(status);
                return Ok(self);
            };
            debug_log!("HTTP Context: Running endpoint handler");
            let result = endpoint.run(self).await.map_err(|e| e.boxed());
            debug_log!("HTTP Context: Handler completed");
            result

        } else {
            debug_log!("HTTP Context: No endpoint available (client context)");
            // No endpoint available (client context)
            Ok(self)
        }
    }


    /// Checks whether the request fulfills the endpoint's security requirements.
    ///
    /// Returns `Ok(())` if the request passes all checks, or `Err(HttpError)` with
    /// the appropriate error variant if a check fails.
    pub fn request_check(
        &mut self,
        endpoint: &Arc<UrlNode<HttpContext<TS>, TS>>,
    ) -> Result<(), HttpError> {
        let config = endpoint.get_params::<HttpSafety>().unwrap_or_default();
        // println!(
        //     "Checking request: {:?} {}{} ",config,self.request.meta.method(),config.check_method(&self.request.meta.method())
        // );
        if !config.check_body_size(self.request.meta.get_content_length().unwrap_or(0)) {
            return Err(HttpError::PayloadTooLarge);
        }
        if !config.check_method(&self.request.meta.method()) {
            return Err(HttpError::MethodNotAllowed);
        }
        if !config.check_content_type(&self.request.meta.get_content_type().unwrap_or_default()) {
            return Err(HttpError::UnsupportedMediaType);
        }
        return Ok(());
    }

    /// Returns the meta in the request as reference
    pub fn meta(&mut self) -> &mut HttpMeta {
        &mut self.request.meta
    }

    /// Returns the runtime config if this is a server context.
    pub fn runtime(&self) -> Option<Arc<RuntimeConfig>> {
        match &self.executable {
            Executable::Request { runtime, .. } => Some(runtime.clone()),
            Executable::<TS>::Response => None,
        }
    }

    /// Returns the endpoint URL if this is a server context
    pub fn endpoint(&self) -> Option<Arc<UrlNode<HttpContext<TS>, TS>>> {
        match &self.executable {
            Executable::Request { endpoint, .. } => Some(endpoint.clone()),
            Executable::<TS>::Response => None,
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

    /// Get a path segment by index position, skipping the implicit leading empty
    /// segment produced by the leading `/` in HTTP paths.
    /// For example, in "/api/users/123", segment(0) = "api", segment(1) = "users", segment(2) = "123"
    pub fn segment(&mut self, index: usize) -> String {
        self.request.meta.get_path(index + 1)
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
    pub fn headers(&self) -> &HashMap<String, crate::message::meta::HeaderValue> {
        &self.request.meta.header
    }

    /// Convenience method to get a specific header value.
    pub fn header(&self, key: &str) -> Option<&crate::message::meta::HeaderValue> {
        self.request.meta.header.get(key)
    }

    /// Convenience method to get a header value as a string.
    /// Returns the first value if multiple values exist.
    pub fn header_str(&self, key: &str) -> Option<&str> {
        self.request.meta.header.get(key).and_then(|hv| match hv {
            crate::message::meta::HeaderValue::Single(s) => Some(s.as_str()),
            crate::message::meta::HeaderValue::Multiple(v) => v.first().map(|s| s.as_str()),
        })
    }

    /// Convenience method to check if a header exists.
    pub fn has_header(&self, key: &str) -> bool {
        self.request.meta.header.contains_key(key)
    }

    /// Get the full cookie map
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

impl<TS: TransportSpec> RequestContext for HttpContext<TS> {
    type Request = HttpRequest;
    type Response = HttpResponse;
    type Error = crate::protocol::HttpError;

    fn handle_error(&mut self) {
        match &self.executable {
            Executable::Request { .. } => {
                // Server: return a generic error response.
                // The specific status code is determined by the error type
                // in the protocol-level handle loop via error_response_from().
                self.response = response_templates::html_response(
                    "<h1>500 Internal Server Error</h1><br><p>An unexpected error occurred</p>",
                )
                .status(500);
            }
            Executable::<TS>::Response => {
                // Client: set default error response
                self.response = HttpResponse::default();
            }
        }
    }

    fn role(&self) -> ProtocolRole {
        match &self.executable {
            Executable::Request { .. } => ProtocolRole::Server,
            Executable::<TS>::Response => ProtocolRole::Client,
        }
    }
}

impl<TS: TransportSpec> HttpContext<TS> {
    pub fn bad_request(&mut self) {
        self.handle_error();
    }
}

// Type alias for backward compatibility with client code
pub type HttpResCtx<TS = crate::connection::tcp::TcpTransport> = HttpContext<TS>;

impl<TS: TransportSpec> HttpContext<TS> {
    /// Creates a client context for sending requests (backward compatibility)
    pub fn new_res(config: HttpSafety, host: impl Into<String>) -> Self {
        Self::new_client(host.into(), config)
    }

    /// Sends a request to the given connector target and returns the response.
    ///
    /// This path no longer auto-parses/normalizes host string schemes (`http://`, `https://`)
    /// in `send_request`; it now trusts transport target type.
    ///
    /// Connector creation is explicit: callers provide connector config and this
    /// method constructs `TS::Connector` from that config (no default connector).
    pub async fn send_request<TTarget, TConnectorConfig>(
        target: TTarget,
        connector_config: TConnectorConfig,
        request: HttpRequest,
        safety_config: HttpSafety,
    ) -> Result<HttpResponse, ConnectionError>
    where
        TTarget: Into<<TS::Outbound as Outbound>::ConnectTarget>,
        TS::Outbound: From<TConnectorConfig>,
    {
        let _outbound = TS::Outbound::from(connector_config);
        let wire = <TS::Outbound as Outbound>::connect(target.into())
            .await
            .map_err(ConnectionError::IoError)?;
        let (read, write, _meta) = wire.split();

        let mut reader = BufReader::new(read);
        let mut writer = BufWriter::new(write);

        // Write the HTTP request frame
        Self::write_frame(&mut writer, request).await?;
        // Read the HTTP response frame
        let response = Self::read_next_frame(&safety_config, &mut reader).await?;
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
    pub async fn write_frame<W: AsyncWrite + Unpin>(
        write_stream: &mut W,
        request_frame: HttpRequest,
    ) -> Result<(), ConnectionError> {
        request_frame
            .send(write_stream)
            .await
            .map_err(|e| ConnectionError::IoError(e))?;
        Ok(())
    }

    /// Read an HTTP response frame from the stream
    pub async fn read_next_frame<R: AsyncBufRead + Unpin>(
        config: &HttpSafety,
        read_stream: &mut R,
    ) -> Result<HttpResponse, ConnectionError> {
        Ok(HttpResponse::parse_lazy(read_stream, config, false).await)
    }
}

impl HttpContext<crate::connection::tcp::TcpTransport> {
    /// Backward-compatible helper for string hosts with optional port override.
    ///
    /// This helper keeps old ergonomics (`host + optional port`) and then routes
    /// to the transport-target `send_request` API.
    pub async fn send_request_host<T: Into<String>, TConnectorConfig>(
        host: T,
        port: Option<u16>,
        connector_config: TConnectorConfig,
        mut request: HttpRequest,
        safety_config: HttpSafety,
    ) -> Result<HttpResponse, ConnectionError>
    where
        <crate::connection::tcp::TcpTransport as TransportSpec>::Outbound: From<TConnectorConfig>,
    {
        let host_str = host.into();
        let (is_https, without_scheme) = if host_str.starts_with("https://") {
            (true, host_str.trim_start_matches("https://"))
        } else if host_str.starts_with("http://") {
            (false, host_str.trim_start_matches("http://"))
        } else {
            (false, host_str.as_str())
        };

        if is_https {
            return Err(ConnectionError::Other(
                "HTTPS outbound requests are not supported yet; use http://".to_string(),
            ));
        }

        // Parse optional host:port from input string.
        let mut host_part = without_scheme;
        let mut parsed_port: Option<u16> = None;

        if let Some(colon_pos) = without_scheme.rfind(':') {
            let port_part = &without_scheme[colon_pos + 1..];
            if !port_part.is_empty()
                && port_part.len() <= 5
                && port_part.chars().all(|c| c.is_ascii_digit())
            {
                if let Ok(p) = port_part.parse::<u16>() {
                    parsed_port = Some(p);
                    host_part = &without_scheme[..colon_pos];
                }
            }
        }

        let final_port = port.or(parsed_port).unwrap_or(80);

        // Auto set Host header when absent.
        if request.meta.get_host().is_none() {
            if final_port == 80 && port.is_none() && parsed_port.is_none() {
                request.meta.set_host(Some(host_part.to_string()));
            } else {
                request
                    .meta
                    .set_host(Some(format!("{}:{}", host_part, final_port)));
            }
        }

        let target = format!("{}:{}", host_part, final_port);
        Self::send_request(target, connector_config, request, safety_config).await
    }
}

// #[cfg(test)]
// mod test {
//     use crate::http::{
//         context::HttpResCtx, request::request_templates::get_request, safety::HttpSafety,
//     };
//     #[cfg(feature = "tls")]
//     use hotaru_tls::{TlsClientConfig, TlsConnector, TlsTransport};
//     use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    // =========================================================================
    // Socket Address Accessor Tests
    // =========================================================================

    // #[test]
    // fn test_client_ip_with_address() {
    //     use crate::app::application::App;
    //     use crate::http::request::HttpRequest;
    //     use crate::url::Url;
    //     use std::sync::Arc;

    //     let app = App::new().build();
    //     let endpoint = Arc::new(Url::<super::HttpContext>::default());
    //     let request = HttpRequest::default();
    //     let remote = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 54321);
    //     let local = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);

    //     let ctx = super::HttpContext::new_server(app, endpoint, request, Some(remote), Some(local));

    //     // Test client_ip()
    //     assert_eq!(ctx.client_ip(), Some(remote));
    //     assert_eq!(ctx.client_ip_or_default(), remote);

    //     // Test client_ip_only()
    //     assert_eq!(
    //         ctx.client_ip_only(),
    //         Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)))
    //     );
    //     assert_eq!(
    //         ctx.client_ip_only_or_default(),
    //         IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))
    //     );

    //     // Test server_addr()
    //     assert_eq!(ctx.server_addr(), Some(local));
    //     assert_eq!(ctx.server_addr_or_default(), local);

    //     // Test aliases
    //     assert_eq!(ctx.remote_addr(), Some(remote));
    //     assert_eq!(ctx.remote_addr_or_default(), remote);
    //     assert_eq!(ctx.local_addr(), Some(local));
    //     assert_eq!(ctx.local_addr_or_default(), local);
    // }

//     #[test]
//     fn test_client_ip_without_address() {
//         use crate::app::application::App;
//         use crate::http::request::HttpRequest;
//         use crate::url::Url;
//         use std::sync::Arc;

//         let app = App::new().build();
//         let endpoint = Arc::new(Url::<super::HttpContext>::default());
//         let request = HttpRequest::default();

//         let ctx = super::HttpContext::new_server(
//             app, endpoint, request, None, // No remote address
//             None, // No local address
//         );

//         let unset = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

//         // Test client_ip() returns None
//         assert_eq!(ctx.client_ip(), None);
//         assert_eq!(ctx.client_ip_or_default(), unset);

//         // Test client_ip_only() returns None
//         assert_eq!(ctx.client_ip_only(), None);
//         assert_eq!(
//             ctx.client_ip_only_or_default(),
//             IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
//         );

//         // Test server_addr() returns None
//         assert_eq!(ctx.server_addr(), None);
//         assert_eq!(ctx.server_addr_or_default(), unset);

//         // Test aliases
//         assert_eq!(ctx.remote_addr(), None);
//         assert_eq!(ctx.remote_addr_or_default(), unset);
//         assert_eq!(ctx.local_addr(), None);
//         assert_eq!(ctx.local_addr_or_default(), unset);
//     }

//     #[test]
//     fn test_client_context_has_no_addresses() {
//         let ctx = super::HttpContext::<crate::connection::tcp::TcpTransport>::new_client(
//             "example.com".to_string(),
//             HttpSafety::default(),
//         );

//         // Client contexts start with no addresses
//         assert_eq!(ctx.client_ip(), None);
//         assert_eq!(ctx.server_addr(), None);
//     }

//     #[test]
//     fn test_ipv6_address() {
//         use crate::app::application::App;
//         use crate::http::request::HttpRequest;
//         use crate::url::Url;
//         use std::net::Ipv6Addr;
//         use std::sync::Arc;

//         let app = App::new().build();
//         let endpoint = Arc::new(Url::<super::HttpContext>::default());
//         let request = HttpRequest::default();
//         let remote = SocketAddr::new(
//             IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)),
//             54321,
//         );
//         let local = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 8080);

//         let ctx = super::HttpContext::new_server(app, endpoint, request, Some(remote), Some(local));

//         assert_eq!(ctx.client_ip(), Some(remote));
//         assert_eq!(
//             ctx.client_ip_only(),
//             Some(IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)))
//         );
//         assert_eq!(ctx.server_addr(), Some(local));
//     }

//     // =========================================================================
//     // HTTP Client Tests
//     // =========================================================================

//     // #[tokio::test]
//     // async fn request_a_page() {
//     //     let builder = ConnectionBuilder::new("example.com", 443)
//     //         .protocol(Protocol::HTTP)
//     //         .tls(true);
//     //     let connection = builder.connect().await.unwrap();
//     //     let mut request = HttpResCtx::new(
//     //         connection,
//     //         HttpSafety::new().with_max_body_size(25565),
//     //         "example.com",
//     //     );
//     //     let _ = request.process(request_templates::get_request("/")).await;
//     //     request.parse_response().await;
//     //     // println!("{:?}, {:?}", request.response.meta, request.response.body);
//     // }

//     /// HTTPS test (requires `tls` feature and external network).
//     #[cfg(feature = "tls")]
//     #[tokio::test]
//     #[ignore = "requires external network and TLS feature"]
//     async fn request_another_page() {
//         let connector = TlsConnector::new(TlsClientConfig::new()).unwrap();
//         let response = HttpResCtx::<TlsTransport>::send_request(
//             ("api.pmine.org".to_string(), 443),
//             connector,
//             get_request("/num/change/lhsduifhsjdbczfjgszjdhfgxyjey/36/2"),
//             HttpSafety::new().with_max_body_size(25565),
//         )
//         .await
//         .unwrap();
//         println!("{:?}, {:?}", response.meta, response.body);
//     }

//     /// HTTPS chunked-response test (requires `tls` feature and external network).
//     #[cfg(feature = "tls")]
//     #[tokio::test]
//     #[ignore = "requires external network and TLS feature"]
//     async fn request_chunked_page() {
//         let connector = TlsConnector::new(TlsClientConfig::new()).unwrap();
//         let response = HttpResCtx::<TlsTransport>::send_request(
//             ("api.pmine.org".to_string(), 443),
//             connector,
//             get_request("/num/c2"),
//             HttpSafety::new().with_max_body_size(25565),
//         )
//         .await
//         .unwrap();
//         println!("{:?}, {:?}", response.meta, response.body);
//     }

//     /// Test requires a local server running on port 3003
//     /// Run with: cargo test --lib -- --ignored localhost
//     #[tokio::test]
//     #[ignore = "requires local server on port 3003"]
//     async fn localhost() {
//         let response = HttpResCtx::send_request_host(
//             "http://localhost:3003",
//             None,
//             crate::connection::tcp::TcpConnector,
//             get_request("/"),
//             HttpSafety::new().with_max_body_size(25565),
//         )
//         .await
//         .unwrap();
//         println!("{:?}, {:?}", response.meta, response.body);
//     }
// }
