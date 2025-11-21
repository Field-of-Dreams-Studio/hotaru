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
use crate::debug_log; 
use webpki_roots::TLS_SERVER_ROOTS;

use crate::connection::error::{ConnectionError, Result}; 
use super::stream::TcpConnectionStream; 

use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::Path;

/// Protocol to use for database connections
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
            Self::Postgres => write!(f, "postgres"), // PostgreSQL protocol 
            Self::MySQL => write!(f, "mysql"), // MySQL protocol 
            Self::MongoDB => write!(f, "mongodb"), // MongoDB protocol 
            Self::Redis => write!(f, "redis"), // Redis protocol 
            Self::HTTP => write!(f, "http"), // HTTP scheme 
            Self::WebSocket => write!(f, "ws"),  // WebSocket schemes 
            Self::Custom => write!(f, "custom"),
        }
    }
} 

/// Authentication options for database connections
#[derive(Debug, Clone)]
pub enum Authentication {
    None,
    UsernamePassword(String, String),
    Token(String),
    Certificate(Vec<u8>, Vec<u8>), // cert, key 
    Custom(Arc<dyn std::any::Any + Send + Sync>),
} 

/// Builder for creating database connections
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
    /// Create a new connection builder with default settings
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


    /// Enable or disable TLS encryption
    pub fn tls(mut self, enable: bool) -> Self {
        self.use_tls = enable;
        self
    }

    /// Set the protocol to use
    pub fn protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        // Automatically set default port if not specified
        match protocol {
            Protocol::Postgres if self.port == 0 => self.port = 5432,
            Protocol::MySQL if self.port == 0 => self.port = 3306,
            Protocol::MongoDB if self.port == 0 => self.port = 27017,
            Protocol::Redis if self.port == 0 => self.port = 6379,
            Protocol::HTTP if self.port == 0 => self.port = if self.use_tls { 443 } else { 80 },
            Protocol::WebSocket if self.port == 0 => self.port = if self.use_tls { 443 } else { 80 }, 
            _ => {}
        }
        self
    }

    /// Set authentication credentials
    pub fn auth(mut self, auth: Authentication) -> Self {
        self.auth = auth;
        self
    }

    /// Set username and password for authentication
    pub fn credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.auth = Authentication::UsernamePassword(username.into(), password.into());
        self
    }

    /// Set database name
    pub fn database(mut self, db_name: impl Into<String>) -> Self {
        self.database = Some(db_name.into());
        self
    }

    /// Set maximum time to wait for connection establishment
    pub fn max_connection_time(mut self, duration: Duration) -> Self {
        self.max_connection_time = duration;
        self
    }

    /// Set number of retry attempts
    pub fn retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = attempts;
        self
    }

    /// Set delay between retry attempts
    pub fn retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Set query timeout
    pub fn query_timeout(mut self, timeout: Duration) -> Self {
        self.query_timeout = timeout;
        self
    }

    /// Add additional parameters
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

    /// Load a custom root CA certificate from a PEM file for TLS connections
    pub fn root_certificate(mut self, path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(path.as_ref()).map_err(ConnectionError::IoError)?;
        let mut buf = Vec::new();
        BufReader::new(file).read_to_end(&mut buf).map_err(ConnectionError::IoError)?;
        self.root_cert_pem = Some(buf);
        Ok(self)
    }

    /// Create connection URL based on config
    pub fn url(&self) -> String {
        let auth_str = match &self.auth {
            Authentication::UsernamePassword(user, pass) => format!("{}:{}@", user, pass),
            Authentication::Token(token) => format!("token:{}@", token),
            _ => String::new(),
        };
        
        // Get the schem string 
        let scheme = match self.protocol {
            Protocol::Postgres => if self.use_tls { "postgresql+ssl" } else { "postgresql" },
            Protocol::MySQL => if self.use_tls { "mysql+ssl" } else { "mysql" },
            Protocol::MongoDB => if self.use_tls { "mongodb+ssl" } else { "mongodb" },
            Protocol::Redis => if self.use_tls { "redis+ssl" } else { "redis" },
            Protocol::HTTP => if self.use_tls { "https" } else { "http" },
            Protocol::WebSocket => if self.use_tls { "wss" } else { "ws" }, 
            Protocol::Custom => if self.use_tls { "tls" } else { "tcp" },
        };
        
        // Path or database construction based on protocol type
        let path_or_db = match self.protocol {
            Protocol::HTTP | Protocol::WebSocket => &self.path, // Both HTTP and WebSocket use paths
            _ => self.database.as_ref().map_or("", |db| db),    // Database protocols use database names
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

    /// Establish a connection with retry logic
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
        // 1) TCP
        let addr = format!("{}:{}", self.host, self.port);
        let tcp = tokio::time::timeout(
            self.max_connection_time, TcpStream::connect(&addr)
        )
        .await??;

        if !self.use_tls {
            return Ok(TcpConnectionStream::Tcp(tcp));
        }

        // 2) TLS root store
        let mut root_store = RootCertStore::empty();
        // Create an empty RootCertStore, which will hold trusted root certificates for TLS.

        if let Some(pem) = &self.root_cert_pem {
            // If the user provided custom root certificates in PEM format, proceed.
            let mut reader = BufReader::new(Cursor::new(pem));
            // Wrap the PEM bytes in a Cursor, then a BufReader for efficient reading.

            let certs = rustls_pemfile::read_all(&mut reader)
                .into_iter()
                .filter_map(|item| match item {
                    Ok(Item::X509Certificate(cert)) => Some(cert),
                    _ => None,
                }); 
            // Parse all items in the PEM data, keep only X.509 certificates (ignore keys, etc).

            let (added, ignored) = root_store.add_parsable_certificates(certs);
            // Add the parsed certificates to the root store. Returns count of added and ignored certs.
            debug_log!("Added {} certificates, ignored {}", added, ignored);
            // Print info about how many certificates were successfully added.
        } else { 
            root_store.extend(TLS_SERVER_ROOTS.iter().cloned()); 
        }

        // 3) Build a client config (no client auth)
        let provider = Arc::new(default_provider());
        let config = ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|e| ConnectionError::TlsError(e.to_string()))?
            .with_root_certificates(root_store)
            .with_no_client_auth();

        // 4) Hand-shake
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
