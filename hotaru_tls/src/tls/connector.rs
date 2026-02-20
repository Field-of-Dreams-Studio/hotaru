//! TLS client-side connector.

use std::sync::Arc;
use async_trait::async_trait;
use tokio::net::TcpStream;

use hotaru_core::connection::Connector;

use crate::config::client::TlsClientConfig;
use super::stream::TlsStream;

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

#[async_trait]
impl Connector for TlsConnector {
    type Stream = TlsStream;
    type Target = (String, u16); // (hostname, port)

    async fn connect(&self, target: Self::Target) -> std::io::Result<Self::Stream> {
        let (host, port) = target;

        let tcp = TcpStream::connect(format!("{}:{}", host, port)).await?;

        let server_name = rustls::pki_types::ServerName::try_from(host.as_str())
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid hostname: {}", host),
                )
            })?
            .to_owned();

        self.connector
            .connect(server_name, tcp)
            .await
            .map(TlsStream::Client)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
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
