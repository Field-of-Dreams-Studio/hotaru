//! ConnectionBuilder — runtime TCP/TLS connection factory.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use rustls_pemfile::Item;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use rustls::{ClientConfig, RootCertStore, pki_types::ServerName};
use rustls::crypto::ring::default_provider;
use webpki_roots::TLS_SERVER_ROOTS;

use hotaru_core::debug_log;
use hotaru_core::connection::error::{ConnectionError, Result};

use super::stream::TcpOrTlsStream;

use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::Path;

/// Protocol to use for outbound connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Postgres,
    MySQL,
    MongoDB,
    Redis,
    HTTP,
    WebSocket,
    Custom,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Postgres  => write!(f, "postgres"),
            Self::MySQL     => write!(f, "mysql"),
            Self::MongoDB   => write!(f, "mongodb"),
            Self::Redis     => write!(f, "redis"),
            Self::HTTP      => write!(f, "http"),
            Self::WebSocket => write!(f, "ws"),
            Self::Custom    => write!(f, "custom"),
        }
    }
}

/// Authentication options for outbound connections.
#[derive(Debug, Clone)]
pub enum Authentication {
    None,
    UsernamePassword(String, String),
    Token(String),
    Certificate(Vec<u8>, Vec<u8>),
    Custom(Arc<dyn std::any::Any + Send + Sync>),
}

/// Builder for creating outbound TCP or TLS connections.
///
/// Use when the transport (TCP vs TLS) is decided at runtime based on
/// configuration. For compile-time static TLS, prefer `TlsConnector` +
/// `TransportSpec` instead.
#[derive(Debug, Clone)]
pub struct ConnectionBuilder {
    host: String,
    port: u16,
    use_tls: bool,
    protocol: Protocol,
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

impl ConnectionBuilder {
    /// Create a new connection builder with default settings.
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            use_tls: false,
            protocol: Protocol::Custom,
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

    pub fn tls(mut self, enable: bool) -> Self {
        self.use_tls = enable;
        self
    }

    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        match protocol {
            Protocol::Postgres  if self.port == 0 => self.port = 5432,
            Protocol::MySQL     if self.port == 0 => self.port = 3306,
            Protocol::MongoDB   if self.port == 0 => self.port = 27017,
            Protocol::Redis     if self.port == 0 => self.port = 6379,
            Protocol::HTTP      if self.port == 0 => self.port = if self.use_tls { 443 } else { 80 },
            Protocol::WebSocket if self.port == 0 => self.port = if self.use_tls { 443 } else { 80 },
            _ => {}
        }
        self
    }

    pub fn auth(mut self, auth: Authentication) -> Self {
        self.auth = auth;
        self
    }

    pub fn credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.auth = Authentication::UsernamePassword(username.into(), password.into());
        self
    }

    pub fn database(mut self, db_name: impl Into<String>) -> Self {
        self.database = Some(db_name.into());
        self
    }

    pub fn max_connection_time(mut self, duration: Duration) -> Self {
        self.max_connection_time = duration;
        self
    }

    pub fn retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = attempts;
        self
    }

    pub fn retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    pub fn query_timeout(mut self, timeout: Duration) -> Self {
        self.query_timeout = timeout;
        self
    }

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
        let file = File::open(path.as_ref()).map_err(ConnectionError::IoError)?;
        let mut buf = Vec::new();
        BufReader::new(file).read_to_end(&mut buf).map_err(ConnectionError::IoError)?;
        self.root_cert_pem = Some(buf);
        Ok(self)
    }

    /// Build the connection URL from the current configuration.
    pub fn url(&self) -> String {
        let auth_str = match &self.auth {
            Authentication::UsernamePassword(user, pass) => format!("{}:{}@", user, pass),
            Authentication::Token(token) => format!("token:{}@", token),
            _ => String::new(),
        };

        let scheme = match self.protocol {
            Protocol::Postgres  => if self.use_tls { "postgresql+ssl" } else { "postgresql" },
            Protocol::MySQL     => if self.use_tls { "mysql+ssl"       } else { "mysql"       },
            Protocol::MongoDB   => if self.use_tls { "mongodb+ssl"     } else { "mongodb"     },
            Protocol::Redis     => if self.use_tls { "redis+ssl"       } else { "redis"       },
            Protocol::HTTP      => if self.use_tls { "https"           } else { "http"        },
            Protocol::WebSocket => if self.use_tls { "wss"             } else { "ws"          },
            Protocol::Custom    => if self.use_tls { "tls"             } else { "tcp"         },
        };

        let path_or_db = match self.protocol {
            Protocol::HTTP | Protocol::WebSocket => &self.path,
            _ => self.database.as_ref().map_or("", |db| db),
        };

        let mut params = String::new();
        if !self.additional_params.is_empty() {
            params.push('?');
            for (i, (k, v)) in self.additional_params.iter().enumerate() {
                if i > 0 { params.push('&'); }
                params.push_str(&format!("{}={}", k, v));
            }
        }

        format!("{}://{}{}:{}{}{}", scheme, auth_str, self.host, self.port, path_or_db, params)
    }

    /// Establish a connection with retry logic.
    pub async fn connect(&self) -> Result<TcpOrTlsStream> {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts <= self.retry_attempts {
            match self.try_connect().await {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    last_error = Some(e);
                    if attempts == self.retry_attempts { break; }
                    attempts += 1;
                    tokio::time::sleep(self.retry_delay).await;
                }
            }
        }

        Err(last_error.unwrap_or(ConnectionError::ConnectionRefused))
    }

    async fn try_connect(&self) -> Result<TcpOrTlsStream> {
        let addr = format!("{}:{}", self.host, self.port);
        let tcp = tokio::time::timeout(
            self.max_connection_time,
            TcpStream::connect(&addr),
        )
        .await??;

        if !self.use_tls {
            return Ok(TcpOrTlsStream::Tcp(tcp));
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

        let tls = connector
            .connect(server_name, tcp)
            .await
            .map_err(|e| ConnectionError::TlsError(e.to_string()))?;

        Ok(TcpOrTlsStream::Tls(tls))
    }
}
