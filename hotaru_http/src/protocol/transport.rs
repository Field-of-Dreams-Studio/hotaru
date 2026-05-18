//! HTTP transport state — connection-level metadata for HTTP/1.1.
//!
//! Tracks keep-alive status, socket addresses, request count, and safety
//! configuration for a single HTTP connection.

use std::net::SocketAddr;

use crate::message::request::HttpRequest;
use crate::security::safety::HttpSafety;
use hotaru_core::protocol::ProtocolRole;

/// Placeholder address for uninitialized connections.
const UNSET_ADDR: SocketAddr =
    SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)), 0);

/// HTTP transport state.
///
/// Tracks connection-level state for HTTP:
/// - Connection ID for logging
/// - Keep-alive status
/// - Request count for connection reuse
/// - Safety configuration
#[derive(Clone)]
pub struct HttpTransport {
    /// Unique connection identifier
    id: i128,

    /// Whether this connection supports keep-alive
    pub keep_alive: bool,

    /// Local address of the connection
    pub local_addr: SocketAddr,

    /// Remote address of the connection
    pub remote_addr: SocketAddr,

    /// Number of requests processed on this connection
    pub request_count: u64,

    /// HTTP safety configuration (limits, timeouts, etc.)
    pub safety: HttpSafety,

    /// Role of this protocol instance
    pub role: ProtocolRole,
}

impl HttpTransport {
    /// Creates a new HTTP/1.1 transport with socket addresses.
    pub fn new(
        role: ProtocolRole,
        safety: HttpSafety,
        local_addr: SocketAddr,
        remote_addr: SocketAddr,
    ) -> Self {
        Self {
            id: generate_connection_id(),
            keep_alive: true,
            local_addr,
            remote_addr,
            request_count: 0,
            safety,
            role,
        }
    }

    /// Creates a new HTTP/1.1 transport without socket addresses.
    /// Addresses should be set via set_addresses() when available.
    pub fn new_unbound(role: ProtocolRole, safety: HttpSafety) -> Self {
        Self::new(role, safety, UNSET_ADDR, UNSET_ADDR)
    }

    /// Sets the socket addresses from connection metadata.
    pub fn set_addresses(&mut self, local: Option<SocketAddr>, remote: Option<SocketAddr>) {
        if let Some(addr) = local {
            self.local_addr = addr;
        }
        if let Some(addr) = remote {
            self.remote_addr = addr;
        }
    }

    /// Increments the request counter.
    pub fn increment_requests(&mut self) {
        self.request_count += 1;
    }

    /// Checks if the connection should be kept alive.
    pub fn should_keep_alive(&self) -> bool {
        self.keep_alive
    }

    /// Updates keep-alive based on request headers.
    pub fn update_keep_alive(&mut self, request: &HttpRequest) {
        // Check Connection header
        if let Some(connection) = request.meta.header.get("connection") {
            self.keep_alive = connection.as_str().to_lowercase() != "close";
        } else {
            // HTTP/1.1 defaults to keep-alive
            self.keep_alive = true;
        }
    }
}

/// Generates a unique connection ID.
fn generate_connection_id() -> i128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as i128
}
