use std::marker::PhantomData;
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
use super::protocol::Protocol;
use super::stream::TcpConnectionStream; 

use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::Path;
use av::ver;

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
pub struct ConnectionBuilder<P: Protocol> {
    host: String,
    port: Option<u16>,
    use_tls: bool,
    _protocol: PhantomData<P>,
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

impl<P: Protocol> ConnectionBuilder<P> { 
    /// Create a new connection builder with default settings
    #[ver(update, since = "0.8.0", note = "Generic ConnectionBuilder with protocol default ports")]
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: None,
            use_tls: false,
            _protocol: PhantomData,
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

    /// Set a specific port for the connection
    #[ver(update, since = "0.8.0", note = "Explicit port setter for generic builder")]
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Enable or disable TLS encryption
    pub fn tls(mut self, enable: bool) -> Self {
        self.use_tls = enable;
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

    fn resolved_port(&self) -> Result<u16> {
        self.port
            .or_else(|| P::default_port(self.use_tls))
            .ok_or(ConnectionError::PortRequired)
    }

    /// Establish a connection with retry logic
    pub async fn connect(&self) -> Result<TcpConnectionStream> {
        let port = self.resolved_port()?;
        let mut attempts = 0;
        let mut last_error = None;

        while attempts <= self.retry_attempts {
            match self.try_connect(port).await {
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

        
    async fn try_connect(&self, port: u16) -> Result<TcpConnectionStream> {
        // 1) TCP
        let addr = format!("{}:{}", self.host, port);
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
