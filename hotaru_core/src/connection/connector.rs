use async_trait::async_trait;

use super::stream::ConnStream;

/// Establishes outbound connections and upgrades them into protocol-specific streams.
///
/// Why `connect` takes `&self`:
/// - Connectors can hold reusable runtime state (TLS client config, trust store,
///   SNI policy, connection limits, telemetry).
/// - Callers only pass a target; connector internals decide how to connect.
///
/// Where to store configuration:
/// - Put stable configuration inside the connector struct.
/// - For mutable/reloadable settings, keep shared state inside the connector
///   (`Arc`, `RwLock`, `ArcSwap`) and consult it during `connect`.
#[async_trait]
pub trait Connector: Send + Sync + 'static {
    /// Stream type produced by this connector.
    type Stream: ConnStream;
    /// Target for the outbound connection (host/port, socket addr, URL, etc).
    type Target;

    /// Connect and upgrade to the configured stream type.
    async fn connect(&self, target: Self::Target) -> std::io::Result<Self::Stream>;
}
