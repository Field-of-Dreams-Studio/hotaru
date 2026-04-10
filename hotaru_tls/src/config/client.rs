//! TLS configuration for client-side connections.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

use rustls::crypto::ring::default_provider;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::{ClientConfig, RootCertStore};
use webpki_roots::TLS_SERVER_ROOTS;

/// TLS configuration for client-side connections.
///
/// This configuration specifies how the client validates server certificates
/// and optionally presents its own certificate for mutual TLS.
pub struct TlsClientConfig {
    /// Custom root certificates to trust (in addition to or instead of system roots)
    pub(crate) root_certs: Option<Arc<RootCertStore>>,

    /// Whether to use system root certificates
    pub(crate) use_webpki_roots: bool,

    /// Client certificate and key for mutual TLS
    pub(crate) client_auth: Option<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)>,

    /// ALPN protocols to request (e.g., ["h2", "http/1.1"])
    pub(crate) alpn_protocols: Vec<Vec<u8>>,

    /// Whether to verify the server certificate (disable for testing only!)
    pub(crate) verify_server: bool,
}

impl Clone for TlsClientConfig {
    fn clone(&self) -> Self {
        Self {
            root_certs: self.root_certs.clone(),
            use_webpki_roots: self.use_webpki_roots,
            client_auth: self
                .client_auth
                .as_ref()
                .map(|(certs, key)| (certs.clone(), key.clone_key())),
            alpn_protocols: self.alpn_protocols.clone(),
            verify_server: self.verify_server,
        }
    }
}

impl Default for TlsClientConfig {
    fn default() -> Self {
        Self {
            root_certs: None,
            use_webpki_roots: true,
            client_auth: None,
            alpn_protocols: Vec::new(),
            verify_server: true,
        }
    }
}

impl TlsClientConfig {
    /// Create a new TLS client configuration with default settings (webpki roots).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder for constructing TLS client configuration.
    pub fn builder() -> TlsClientConfigBuilder {
        TlsClientConfigBuilder::default()
    }

    /// Build a rustls `ClientConfig` from this configuration.
    pub(crate) fn build_client_config(&self) -> Result<ClientConfig, TlsClientConfigError> {
        let provider = Arc::new(default_provider());

        // Build root certificate store
        let mut root_store = RootCertStore::empty();

        // Add webpki roots if enabled
        if self.use_webpki_roots {
            root_store.extend(TLS_SERVER_ROOTS.iter().cloned());
        }

        // Add custom roots if provided
        if let Some(custom_roots) = &self.root_certs {
            root_store.extend(custom_roots.roots.iter().cloned());
        }

        let mut builder = ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|e| TlsClientConfigError::InvalidConfig(e.to_string()))?
            .with_root_certificates(root_store);

        // Configure client authentication
        let mut config = if let Some((cert_chain, key)) = &self.client_auth {
            builder
                .with_client_auth_cert(cert_chain.clone(), key.clone_key())
                .map_err(|e| TlsClientConfigError::InvalidCertificate(e.to_string()))?
        } else {
            builder.with_no_client_auth()
        };

        // Set ALPN protocols if configured
        if !self.alpn_protocols.is_empty() {
            config.alpn_protocols = self.alpn_protocols.clone();
        }

        // Disable certificate verification if requested (DANGEROUS!)
        if !self.verify_server {
            config
                .dangerous()
                .set_certificate_verifier(Arc::new(NoCertificateVerification));
        }

        Ok(config)
    }
}

/// Builder for TLS client configuration.
#[derive(Default)]
pub struct TlsClientConfigBuilder {
    root_certs: Option<Arc<RootCertStore>>,
    use_webpki_roots: bool,
    client_cert_chain: Option<Vec<CertificateDer<'static>>>,
    client_private_key: Option<PrivateKeyDer<'static>>,
    alpn_protocols: Vec<Vec<u8>>,
    verify_server: bool,
}

impl TlsClientConfigBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            use_webpki_roots: true,
            verify_server: true,
            ..Default::default()
        }
    }

    /// Add a custom root CA certificate from a PEM file.
    ///
    /// This certificate will be trusted in addition to the webpki roots.
    pub fn add_root_certificate(
        mut self,
        path: impl AsRef<Path>,
    ) -> Result<Self, TlsClientConfigError> {
        let root_store = Self::load_root_certs(path)?;

        self.root_certs = Some(match self.root_certs.take() {
            Some(existing) => {
                let mut merged = RootCertStore::empty();
                merged.extend(existing.roots.iter().cloned());
                merged.extend(root_store.roots.iter().cloned());
                Arc::new(merged)
            }
            None => root_store,
        });

        Ok(self)
    }

    /// Disable webpki roots (system/Mozilla root certificates).
    ///
    /// Use this if you want to ONLY trust custom root certificates.
    pub fn disable_webpki_roots(mut self) -> Self {
        self.use_webpki_roots = false;
        self
    }

    /// Set client certificate for mutual TLS.
    ///
    /// # Arguments
    /// * `cert_path` - Path to client certificate PEM file
    /// * `key_path` - Path to client private key PEM file
    pub fn client_auth(
        mut self,
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
    ) -> Result<Self, TlsClientConfigError> {
        // Load certificate chain
        let file = File::open(cert_path.as_ref()).map_err(|e| TlsClientConfigError::IoError(e))?;
        let mut reader = BufReader::new(file);

        let certs = rustls_pemfile::certs(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TlsClientConfigError::InvalidCertificate(e.to_string()))?;

        if certs.is_empty() {
            return Err(TlsClientConfigError::InvalidCertificate(
                "No certificates found in file".into(),
            ));
        }

        // Load private key
        let file = File::open(key_path.as_ref()).map_err(|e| TlsClientConfigError::IoError(e))?;
        let mut reader = BufReader::new(file);

        let key = rustls_pemfile::private_key(&mut reader)
            .map_err(|e| TlsClientConfigError::InvalidKey(e.to_string()))?
            .ok_or_else(|| TlsClientConfigError::InvalidKey("No private key found".into()))?;

        self.client_cert_chain = Some(certs);
        self.client_private_key = Some(key);
        Ok(self)
    }

    /// Add an ALPN protocol (e.g., "h2" for HTTP/2, "http/1.1" for HTTP/1.1).
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

    /// Disable server certificate verification (DANGEROUS - testing only!).
    ///
    /// **WARNING**: This disables all certificate validation and should NEVER
    /// be used in production. It makes your connection vulnerable to MITM attacks.
    pub fn danger_disable_verification(mut self) -> Self {
        self.verify_server = false;
        self
    }

    /// Build the TLS client configuration.
    pub fn build(self) -> Result<TlsClientConfig, TlsClientConfigError> {
        let client_auth = match (self.client_cert_chain, self.client_private_key) {
            (Some(chain), Some(key)) => Some((chain, key)),
            (None, None) => None,
            _ => {
                return Err(TlsClientConfigError::MissingField(
                    "both client certificate and key required for mTLS",
                ));
            }
        };

        Ok(TlsClientConfig {
            root_certs: self.root_certs,
            use_webpki_roots: self.use_webpki_roots,
            client_auth,
            alpn_protocols: self.alpn_protocols,
            verify_server: self.verify_server,
        })
    }

    /// Helper to load root certificates from a PEM file.
    fn load_root_certs(path: impl AsRef<Path>) -> Result<Arc<RootCertStore>, TlsClientConfigError> {
        let file = File::open(path.as_ref()).map_err(|e| TlsClientConfigError::IoError(e))?;
        let mut reader = BufReader::new(file);

        let certs = rustls_pemfile::certs(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TlsClientConfigError::InvalidCertificate(e.to_string()))?;

        let mut root_store = RootCertStore::empty();
        let (added, _ignored) = root_store.add_parsable_certificates(certs);

        if added == 0 {
            return Err(TlsClientConfigError::InvalidCertificate(
                "No valid CA certificates found".into(),
            ));
        }

        Ok(Arc::new(root_store))
    }
}

/// Certificate verifier that accepts all certificates (DANGEROUS!).
#[derive(Debug)]
struct NoCertificateVerification;

impl rustls::client::danger::ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

/// Errors that can occur during TLS client configuration.
#[derive(Debug)]
pub enum TlsClientConfigError {
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

impl std::fmt::Display for TlsClientConfigError {
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

impl std::error::Error for TlsClientConfigError {}
