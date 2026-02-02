#![allow(deprecated)]

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use rustls_pemfile::Item;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use rustls::{
    ClientConfig, RootCertStore, pki_types::ServerName,
};
use rustls::crypto::ring::default_provider;
use webpki_roots::TLS_SERVER_ROOTS;

use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::Path;

use av::ver;

use crate::debug_log;
use crate::connection::builder::Authentication;
use crate::connection::error::{ConnectionError, Result};
use crate::connection::stream::TcpConnectionStream;

/// Compatibility protocol enum retained for legacy ConnectionBuilder usage.
#[ver(deprecated, since = "0.8.0", note = "Use a Protocol type with ConnectionBuilder::<P>::new(host).port(...)")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyProtocol {
    Postgres,
    MySQL,
    MongoDB,
    Redis,
    HTTP,
    WebSocket,
    Custom,
}

impl fmt::Display for LegacyProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Postgres => write!(f, "postgres"),
            Self::MySQL => write!(f, "mysql"),
            Self::MongoDB => write!(f, "mongodb"),
            Self::Redis => write!(f, "redis"),
            Self::HTTP => write!(f, "http"),
            Self::WebSocket => write!(f, "ws"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

/// Compatibility connection builder retained for legacy APIs.
#[ver(deprecated, since = "0.8.0", note = "Use ConnectionBuilder::<P>::new(host).port(...) and protocol defaults")]
#[derive(Debug, Clone)]
pub struct LegacyConnectionBuilder {
    host: String,
    port: u16,
    use_tls: bool,
    protocol: LegacyProtocol,
    auth: Authentication,
    database: Option<String>,
    max_connection_time: Duration,
    retry_attempts: u32,
    retry_delay: Duration,
    query_timeout: Duration,
    path: String,
    additional_params: std::collections::HashMap<String, String>,
    root_cert_pem: Option<Vec<u8>>,
}

impl LegacyConnectionBuilder {
    /// Create a new connection builder with legacy signature.
    #[ver(deprecated, since = "0.8.0", note = "Use ConnectionBuilder::<P>::new(host).port(...) instead")]
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            use_tls: false,
            protocol: LegacyProtocol::Custom,
            auth: Authentication::None,
            database: None,
            max_connection_time: Duration::from_secs(30),
            retry_attempts: 3,
            retry_delay: Duration::from_millis(500),
            query_timeout: Duration::from_secs(30),
            path: String::new(),
            additional_params: std::collections::HashMap::new(),
            root_cert_pem: None,
        }
    }

    /// Enable or disable TLS encryption.
    pub fn tls(mut self, enable: bool) -> Self {
        self.use_tls = enable;
        self
    }

    /// Set the protocol to use.
    #[ver(deprecated, since = "0.8.0", note = "Use ConnectionBuilder::<P> with Protocol::default_port")]
    pub fn protocol(mut self, protocol: LegacyProtocol) -> Self {
        self.protocol = protocol;
        if self.port == 0 {
            match protocol {
                LegacyProtocol::Postgres => self.port = 5432,
                LegacyProtocol::MySQL => self.port = 3306,
                LegacyProtocol::MongoDB => self.port = 27017,
                LegacyProtocol::Redis => self.port = 6379,
                LegacyProtocol::HTTP => self.port = if self.use_tls { 443 } else { 80 },
                LegacyProtocol::WebSocket => self.port = if self.use_tls { 443 } else { 80 },
                LegacyProtocol::Custom => {}
            }
        }
        self
    }

    /// Set authentication credentials.
    pub fn auth(mut self, auth: Authentication) -> Self {
        self.auth = auth;
        self
    }

    /// Set username and password for authentication.
    pub fn credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.auth = Authentication::UsernamePassword(username.into(), password.into());
        self
    }

    /// Set database name.
    pub fn database(mut self, db_name: impl Into<String>) -> Self {
        self.database = Some(db_name.into());
        self
    }

    /// Set maximum time to wait for connection establishment.
    pub fn max_connection_time(mut self, duration: Duration) -> Self {
        self.max_connection_time = duration;
        self
    }

    /// Set number of retry attempts.
    pub fn retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = attempts;
        self
    }

    /// Set delay between retry attempts.
    pub fn retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Set query timeout.
    pub fn query_timeout(mut self, timeout: Duration) -> Self {
        self.query_timeout = timeout;
        self
    }

    /// Add additional parameters.
    pub fn param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional_params.insert(key.into(), value.into());
        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Load a custom root CA certificate from a PEM file for TLS connections.
    pub fn root_certificate(mut self, path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(path.as_ref()).map_err(ConnectionError::IoError)?;
        let mut buf = Vec::new();
        BufReader::new(file).read_to_end(&mut buf).map_err(ConnectionError::IoError)?;
        self.root_cert_pem = Some(buf);
        Ok(self)
    }

    /// Create connection URL based on legacy config.
    #[ver(deprecated, since = "0.8.0", note = "Construct URLs manually or use protocol-specific builders")]
    pub fn url(&self) -> String {
        let auth_str = match &self.auth {
            Authentication::UsernamePassword(user, pass) => format!("{}:{}@", user, pass),
            Authentication::Token(token) => format!("token:{}@", token),
            _ => String::new(),
        };

        let scheme = match self.protocol {
            LegacyProtocol::Postgres => if self.use_tls { "postgresql+ssl" } else { "postgresql" },
            LegacyProtocol::MySQL => if self.use_tls { "mysql+ssl" } else { "mysql" },
            LegacyProtocol::MongoDB => if self.use_tls { "mongodb+ssl" } else { "mongodb" },
            LegacyProtocol::Redis => if self.use_tls { "redis+ssl" } else { "redis" },
            LegacyProtocol::HTTP => if self.use_tls { "https" } else { "http" },
            LegacyProtocol::WebSocket => if self.use_tls { "wss" } else { "ws" },
            LegacyProtocol::Custom => if self.use_tls { "tls" } else { "tcp" },
        };

        let path_or_db = match self.protocol {
            LegacyProtocol::HTTP | LegacyProtocol::WebSocket => &self.path,
            _ => self.database.as_ref().map_or("", |db| db),
        };

        let mut params = String::new();
        if !self.additional_params.is_empty() {
            params.push('?');
            for (i, (k, v)) in self.additional_params.iter().enumerate() {
                if i > 0 {
                    params.push('&');
                }
                params.push_str(&format!("{}={}", k, v));
            }
        }

        format!("{}://{}{}:{}{}{}", scheme, auth_str, self.host, self.port, path_or_db, params)
    }

    /// Establish a connection with retry logic.
    pub async fn connect(&self) -> Result<TcpConnectionStream> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts <= self.retry_attempts {
            match self.try_connect().await {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    last_error = Some(e);
                    if attempts == self.retry_attempts {
                        break;
                    }

                    attempts += 1;
                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }

        Err(last_error.unwrap_or(ConnectionError::ConnectionRefused))
    }

    async fn try_connect(&self) -> Result<TcpConnectionStream> {
        let addr = format!("{}:{}", self.host, self.port);
        let tcp = tokio::time::timeout(
            self.max_connection_time,
            TcpStream::connect(&addr),
        )
        .await??;

        if !self.use_tls {
            return Ok(TcpConnectionStream::Tcp(tcp));
        }

        let mut root_store = RootCertStore::empty();

        if let Some(pem) = &self.root_cert_pem {
            let mut reader = BufReader::new(Cursor::new(pem));

            let certs = rustls_pemfile::read_all(&mut reader)
                .into_iter()
                .filter_map(|item| match item {
                    Ok(Item::X509Certificate(cert)) => Some(cert),
                    _ => None,
                });

            let (added, ignored) = root_store.add_parsable_certificates(certs);
            debug_log!("Added {} certificates, ignored {}", added, ignored);
        } else {
            root_store.extend(TLS_SERVER_ROOTS.iter().cloned());
        }

        let provider = Arc::new(default_provider());
        let config = ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|e| ConnectionError::TlsError(e.to_string()))?
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(config));
        let server_name = ServerName::try_from(self.host.to_owned())
            .map_err(|_| ConnectionError::HostResolutionFailed(self.host.clone()))?;

        let tls_stream = connector
            .connect(server_name, tcp)
            .await
            .map_err(|e| ConnectionError::TlsError(e.to_string()))?;

        Ok(TcpConnectionStream::Tls(tls_stream))
    }
}

#[ver(deprecated, since = "0.8.0", note = "Use LegacyConnectionBuilder only for temporary compatibility")]
pub type ConnectionBuilder = LegacyConnectionBuilder;

#[ver(deprecated, since = "0.8.0", note = "Use LegacyProtocol only for temporary compatibility")]
pub type Protocol = LegacyProtocol;
