//! TLS configuration for server-side accepters.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

use rustls::crypto::ring::default_provider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};

/// TLS configuration for server-side connections.
///
/// This configuration specifies how the server presents itself to clients
/// and whether it requires client authentication (mutual TLS).
pub struct TlsConfig {
    /// Server certificate chain (server cert + intermediates)
    pub(crate) cert_chain: Vec<CertificateDer<'static>>,

    /// Server private key
    pub(crate) private_key: PrivateKeyDer<'static>,

    /// Client certificate verification mode
    pub(crate) client_auth: ClientAuth,

    /// ALPN protocols to advertise (e.g., ["h2", "http/1.1"])
    pub(crate) alpn_protocols: Vec<Vec<u8>>,
}

/// Client certificate authentication mode.
#[derive(Clone)]
pub enum ClientAuth {
    /// No client authentication required
    None,

    /// Client certificates are optional but will be verified if present
    Optional {
        /// Root certificates trusted for client authentication
        root_certs: Arc<RootCertStore>,
    },

    /// Client certificates are required
    Required {
        /// Root certificates trusted for client authentication
        root_certs: Arc<RootCertStore>,
    },
}

impl Clone for TlsConfig {
    fn clone(&self) -> Self {
        Self {
            cert_chain: self.cert_chain.clone(),
            private_key: self.private_key.clone_key(),
            client_auth: self.client_auth.clone(),
            alpn_protocols: self.alpn_protocols.clone(),
        }
    }
}

impl TlsConfig {
    /// Create a new TLS configuration with the given certificate and key.
    ///
    /// # Arguments
    /// * `cert_chain` - Server certificate chain (leaf cert first, then intermediates)
    /// * `private_key` - Server private key
    ///
    /// # Example
    /// ```no_run
    /// use hotaru_tls::config::TlsConfig;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = TlsConfig::builder()
    ///     .cert_chain_file("server-cert.pem")?
    ///     .private_key_file("server-key.pem")?
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(
        cert_chain: Vec<CertificateDer<'static>>,
        private_key: PrivateKeyDer<'static>,
    ) -> Self {
        Self {
            cert_chain,
            private_key,
            client_auth: ClientAuth::None,
            alpn_protocols: Vec::new(),
        }
    }

    /// Create a builder for constructing TLS configuration.
    pub fn builder() -> TlsConfigBuilder {
        TlsConfigBuilder::default()
    }

    /// Build a rustls `ServerConfig` from this configuration.
    pub(crate) fn build_server_config(&self) -> Result<ServerConfig, TlsConfigError> {
        let provider = Arc::new(default_provider());

        let builder = ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|e| TlsConfigError::InvalidConfig(e.to_string()))?;

        // Configure client authentication
        let server_config = match &self.client_auth {
            ClientAuth::None => builder
                .with_no_client_auth()
                .with_single_cert(self.cert_chain.clone(), self.private_key.clone_key())
                .map_err(|e| TlsConfigError::InvalidCertificate(e.to_string()))?,
            ClientAuth::Optional { root_certs } => {
                let verifier = WebPkiClientVerifier::builder(root_certs.clone())
                    .allow_unauthenticated()
                    .build()
                    .map_err(|e| TlsConfigError::InvalidConfig(e.to_string()))?;

                builder
                    .with_client_cert_verifier(verifier)
                    .with_single_cert(self.cert_chain.clone(), self.private_key.clone_key())
                    .map_err(|e| TlsConfigError::InvalidCertificate(e.to_string()))?
            }
            ClientAuth::Required { root_certs } => {
                let verifier = WebPkiClientVerifier::builder(root_certs.clone())
                    .build()
                    .map_err(|e| TlsConfigError::InvalidConfig(e.to_string()))?;

                builder
                    .with_client_cert_verifier(verifier)
                    .with_single_cert(self.cert_chain.clone(), self.private_key.clone_key())
                    .map_err(|e| TlsConfigError::InvalidCertificate(e.to_string()))?
            }
        };

        // Set ALPN protocols if configured
        let mut config = server_config;
        if !self.alpn_protocols.is_empty() {
            config.alpn_protocols = self.alpn_protocols.clone();
        }

        Ok(config)
    }
}

/// Builder for TLS configuration.
#[derive(Default)]
pub struct TlsConfigBuilder {
    cert_chain: Option<Vec<CertificateDer<'static>>>,
    private_key: Option<PrivateKeyDer<'static>>,
    client_auth: Option<ClientAuth>,
    alpn_protocols: Vec<Vec<u8>>,
}

impl TlsConfigBuilder {
    /// Load certificate chain from a PEM file.
    pub fn cert_chain_file(mut self, path: impl AsRef<Path>) -> Result<Self, TlsConfigError> {
        let file = File::open(path.as_ref()).map_err(|e| TlsConfigError::IoError(e))?;
        let mut reader = BufReader::new(file);

        let certs = rustls_pemfile::certs(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TlsConfigError::InvalidCertificate(e.to_string()))?;

        if certs.is_empty() {
            return Err(TlsConfigError::InvalidCertificate(
                "No certificates found in file".into(),
            ));
        }

        self.cert_chain = Some(certs);
        Ok(self)
    }

    /// Set certificate chain from raw PEM bytes.
    pub fn cert_chain_pem(mut self, pem: &[u8]) -> Result<Self, TlsConfigError> {
        let certs = rustls_pemfile::certs(&mut &pem[..])
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TlsConfigError::InvalidCertificate(e.to_string()))?;

        if certs.is_empty() {
            return Err(TlsConfigError::InvalidCertificate(
                "No certificates found".into(),
            ));
        }

        self.cert_chain = Some(certs);
        Ok(self)
    }

    /// Load private key from a PEM file.
    pub fn private_key_file(mut self, path: impl AsRef<Path>) -> Result<Self, TlsConfigError> {
        let file = File::open(path.as_ref()).map_err(|e| TlsConfigError::IoError(e))?;
        let mut reader = BufReader::new(file);

        let key = rustls_pemfile::private_key(&mut reader)
            .map_err(|e| TlsConfigError::InvalidKey(e.to_string()))?
            .ok_or_else(|| TlsConfigError::InvalidKey("No private key found in file".into()))?;

        self.private_key = Some(key);
        Ok(self)
    }

    /// Set private key from raw PEM bytes.
    pub fn private_key_pem(mut self, pem: &[u8]) -> Result<Self, TlsConfigError> {
        let key = rustls_pemfile::private_key(&mut &pem[..])
            .map_err(|e| TlsConfigError::InvalidKey(e.to_string()))?
            .ok_or_else(|| TlsConfigError::InvalidKey("No private key found".into()))?;

        self.private_key = Some(key);
        Ok(self)
    }

    /// Require client certificates (mutual TLS).
    ///
    /// # Arguments
    /// * `ca_certs_path` - Path to CA certificate(s) that will be trusted for client authentication
    pub fn require_client_auth(
        mut self,
        ca_certs_path: impl AsRef<Path>,
    ) -> Result<Self, TlsConfigError> {
        let root_certs = Self::load_root_certs(ca_certs_path)?;
        self.client_auth = Some(ClientAuth::Required { root_certs });
        Ok(self)
    }

    /// Allow optional client certificates (mutual TLS).
    ///
    /// Clients may present certificates, which will be verified against the provided CA,
    /// but they are not required to do so.
    pub fn optional_client_auth(
        mut self,
        ca_certs_path: impl AsRef<Path>,
    ) -> Result<Self, TlsConfigError> {
        let root_certs = Self::load_root_certs(ca_certs_path)?;
        self.client_auth = Some(ClientAuth::Optional { root_certs });
        Ok(self)
    }

    /// Add an ALPN protocol (e.g., "h2" for HTTP/2, "http/1.1" for HTTP/1.1).
    ///
    /// Protocols are advertised in the order they are added.
    pub fn alpn_protocol(mut self, protocol: impl Into<Vec<u8>>) -> Self {
        self.alpn_protocols.push(protocol.into());
        self
    }

    /// Add multiple ALPN protocols at once.
    pub fn alpn_protocols(mut self, protocols: &[&str]) -> Self {
        for proto in protocols {
            self.alpn_protocols.push(proto.as_bytes().to_vec());
        }
        self
    }

    /// Build the TLS configuration.
    pub fn build(self) -> Result<TlsConfig, TlsConfigError> {
        let cert_chain = self
            .cert_chain
            .ok_or_else(|| TlsConfigError::MissingField("certificate chain"))?;
        let private_key = self
            .private_key
            .ok_or_else(|| TlsConfigError::MissingField("private key"))?;

        Ok(TlsConfig {
            cert_chain,
            private_key,
            client_auth: self.client_auth.unwrap_or(ClientAuth::None),
            alpn_protocols: self.alpn_protocols,
        })
    }

    /// Helper to load root certificates from a PEM file.
    fn load_root_certs(path: impl AsRef<Path>) -> Result<Arc<RootCertStore>, TlsConfigError> {
        let file = File::open(path.as_ref()).map_err(|e| TlsConfigError::IoError(e))?;
        let mut reader = BufReader::new(file);

        let certs = rustls_pemfile::certs(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TlsConfigError::InvalidCertificate(e.to_string()))?;

        let mut root_store = RootCertStore::empty();
        let (added, _ignored) = root_store.add_parsable_certificates(certs);

        if added == 0 {
            return Err(TlsConfigError::InvalidCertificate(
                "No valid CA certificates found".into(),
            ));
        }

        Ok(Arc::new(root_store))
    }
}

/// Errors that can occur during TLS configuration.
#[derive(Debug)]
pub enum TlsConfigError {
    /// I/O error reading certificate or key files
    IoError(std::io::Error),

    /// Invalid certificate format or content
    InvalidCertificate(String),

    /// Invalid private key format or content
    InvalidKey(String),

    /// Invalid TLS configuration
    InvalidConfig(String),

    /// Required field is missing
    MissingField(&'static str),
}

impl std::fmt::Display for TlsConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "I/O error: {}", e),
            Self::InvalidCertificate(e) => write!(f, "Invalid certificate: {}", e),
            Self::InvalidKey(e) => write!(f, "Invalid private key: {}", e),
            Self::InvalidConfig(e) => write!(f, "Invalid TLS configuration: {}", e),
            Self::MissingField(field) => write!(f, "Missing required field: {}", field),
        }
    }
}

impl std::error::Error for TlsConfigError {}
