use async_trait::async_trait;
use tokio::net::TcpStream;

use super::stream::ConnStream;

/// Upgrades a raw TCP stream into a protocol-specific stream type.
///
/// This trait enables pluggable stream upgraders for different encryption
/// and transport layers. Implementations can perform handshakes, encryption
/// negotiation, or simply pass through the raw TCP stream.
///
/// Why `upgrade` takes `&self`:
/// - Acceptors are often stateful at runtime (TLS context, cert resolver,
///   ALPN policy, metrics, connection limits, hot-reload handles).
/// - Per-connection behavior may depend on internal state without requiring
///   call-sites to pass extra arguments every time.
///
/// Where to store configuration:
/// - Put long-lived configuration directly in the accepter struct.
/// - For reloadable config, store an indirection in the accepter
///   (e.g. `Arc`, `RwLock`, `ArcSwap`) and read it during `upgrade`.
#[async_trait]
pub trait Accepter: Send + Sync + 'static {
    /// Stream type produced by this accepter.
    type Stream: ConnStream;

    /// Perform the handshake/upgrade and return the configured stream.
    ///
    /// # Arguments
    /// * `tcp` - The raw TCP stream from the listener
    ///
    /// # Returns
    /// The upgraded stream on success, or an I/O error on failure.
    async fn upgrade(&self, tcp: TcpStream) -> std::io::Result<Self::Stream>;
}
