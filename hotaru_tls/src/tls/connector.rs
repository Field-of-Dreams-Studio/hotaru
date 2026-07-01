//! TLS client-side connector.

use std::sync::Arc;
use tokio::net::TcpStream;

use hotaru_core::connection::Connector;

use super::stream::TlsStream;
use crate::config::client::TlsClientConfig;

/// TLS connector for establishing outbound TLS connections.
///
/// Config is baked in at construction time — never passed per-connection.
///
/// # Example
/// ```no_run
/// use hotaru_tls::tls::connector::TlsConnector;
/// use hotaru_tls::config::client::TlsClientConfig;
/// use hotaru_core::connection::Connector;
///
/// # async fn example() -> std::io::Result<()> {
/// let connector = TlsConnector::new(TlsClientConfig::new())
///     .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
///
/// let stream = connector.connect(("example.com".to_string(), 443)).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct TlsConnector {
    connector: tokio_rustls::TlsConnector,
}

impl TlsConnector {
    /// Create a new TLS connector from the given client configuration.
    pub fn new(config: TlsClientConfig) -> Result<Self, TlsConnectorError> {
        let client_config = config
            .build_client_config()
            .map_err(|e| TlsConnectorError::ConfigError(e.to_string()))?;

        Ok(Self {
            connector: tokio_rustls::TlsConnector::from(Arc::new(client_config)),
        })
    }

    /// Get the underlying rustls `TlsConnector`.
    pub fn inner(&self) -> &tokio_rustls::TlsConnector {
        &self.connector
    }
}

impl Connector for TlsConnector {
    type Stream = TlsStream;
    type Target = (String, u16); // (hostname, port)
    type Error = TlsConnectError;

    async fn connect(&self, target: Self::Target) -> Result<Self::Stream, Self::Error> {
        let (host, port) = target;

        // Three distinct failure paths — each gets its own variant so
        // callers can tell whether to retry, fail over, or bail.
        let tcp = TcpStream::connect(format!("{}:{}", host, port))
            .await
            .map_err(TlsConnectError::TcpConnect)?;

        let server_name = rustls::pki_types::ServerName::try_from(host.as_str())
            .map_err(|_| TlsConnectError::InvalidHostname(host.clone()))?
            .to_owned();

        self.connector
            .connect(server_name, tcp)
            .await
            .map(TlsStream::Client)
            .map_err(TlsConnectError::Handshake)
    }
}

/// Errors returned by [`TlsConnector::connect`].
#[derive(Debug)]
pub enum TlsConnectError {
    /// TCP dial failed before TLS could begin.
    TcpConnect(std::io::Error),
    /// Target hostname could not be parsed as a valid SNI name.
    InvalidHostname(String),
    /// TLS handshake failed. Inner `io::Error` carries the rustls
    /// diagnostic verbatim.
    Handshake(std::io::Error),
}

impl std::fmt::Display for TlsConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TcpConnect(e) => write!(f, "TCP connect failed: {}", e),
            Self::InvalidHostname(h) => write!(f, "invalid hostname: {}", h),
            Self::Handshake(e) => write!(f, "TLS handshake failed: {}", e),
        }
    }
}

impl std::error::Error for TlsConnectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TcpConnect(e) | Self::Handshake(e) => Some(e),
            Self::InvalidHostname(_) => None,
        }
    }
}

impl From<TlsConnectError> for std::io::Error {
    /// Lets `TlsOutbound::connect` propagate via `?`. `TcpConnect` and
    /// `Handshake` unwrap losslessly; `InvalidHostname` synthesises an
    /// `ErrorKind::InvalidInput`.
    fn from(err: TlsConnectError) -> Self {
        match err {
            TlsConnectError::TcpConnect(e) | TlsConnectError::Handshake(e) => e,
            TlsConnectError::InvalidHostname(h) => std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("invalid hostname: {}", h),
            ),
        }
    }
}

/// Errors that can occur when creating a `TlsConnector`.
#[derive(Debug)]
pub enum TlsConnectorError {
    ConfigError(String),
}

impl std::fmt::Display for TlsConnectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigError(e) => write!(f, "TLS connector configuration error: {}", e),
        }
    }
}

impl std::error::Error for TlsConnectorError {}
