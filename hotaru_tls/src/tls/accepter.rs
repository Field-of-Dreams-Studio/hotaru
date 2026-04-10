//! TLS server-side accepter.

use async_trait::async_trait;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor as RustlsAcceptor;

use hotaru_core::connection::Accepter;

use super::stream::TlsStream;
use crate::config::server::TlsConfig;

/// TLS accepter that performs the server-side TLS handshake on incoming TCP connections.
///
/// Config is baked in at construction time — never passed per-connection.
///
/// # Example
/// ```no_run
/// use hotaru_tls::tls::accepter::TlsAccepter;
/// use hotaru_tls::config::server::TlsConfig;
/// use hotaru_core::connection::Accepter;
/// use tokio::net::TcpListener;
///
/// # async fn example() -> std::io::Result<()> {
/// let tls_config = TlsConfig::builder()
///     .cert_chain_file("server-cert.pem")?
///     .private_key_file("server-key.pem")?
///     .alpn_protocols(&["h2", "http/1.1"])
///     .build()
///     .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
///
/// let accepter = TlsAccepter::new(tls_config)
///     .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
///
/// let listener = TcpListener::bind("0.0.0.0:443").await?;
/// loop {
///     let (tcp, _) = listener.accept().await?;
///     let stream = accepter.upgrade(tcp).await?;
/// }
/// # }
/// ```
#[derive(Clone)]
pub struct TlsAccepter {
    acceptor: RustlsAcceptor,
}

impl TlsAccepter {
    /// Create a new TLS accepter from the given server configuration.
    pub fn new(config: TlsConfig) -> Result<Self, TlsAccepterError> {
        let server_config = config
            .build_server_config()
            .map_err(|e| TlsAccepterError::ConfigError(e.to_string()))?;

        Ok(Self {
            acceptor: RustlsAcceptor::from(Arc::new(server_config)),
        })
    }

    /// Get the underlying rustls `TlsAcceptor`.
    pub fn inner(&self) -> &RustlsAcceptor {
        &self.acceptor
    }
}

#[async_trait]
impl Accepter for TlsAccepter {
    type Stream = TlsStream;

    async fn upgrade(&self, tcp: TcpStream) -> std::io::Result<Self::Stream> {
        self.acceptor
            .accept(tcp)
            .await
            .map(TlsStream::Server)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

/// Errors that can occur when creating a `TlsAccepter`.
#[derive(Debug)]
pub enum TlsAccepterError {
    ConfigError(String),
}

impl std::fmt::Display for TlsAccepterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigError(e) => write!(f, "TLS accepter configuration error: {}", e),
        }
    }
}

impl std::error::Error for TlsAccepterError {}
