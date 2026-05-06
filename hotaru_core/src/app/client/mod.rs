use std::sync::Arc;

use crate::{
    app::common::{
        AppBuilder, OperationalConfig, RunMode, RuntimeConfig, TimeoutSetting, builder::ClientRole,
    },
    connection::{Outbound, Protocol, TransportSpec},
    url::{UrlError, UrlNode, UrlRoot},
};

pub use registry::ProtocolRegistryKind;

pub mod registry;

/// Outbound runtime for protocol-routed requests.
///
/// `worker` in the shared operational config is interpreted as connection-pool
/// size on the client side, rather than Tokio worker-thread count.
pub struct Client<TS: TransportSpec = crate::connection::tcp::TcpTransport> {
    pub registry: ProtocolRegistryKind<TS>,
    pub target: <TS::Outbound as Outbound>::ConnectTarget,
    pub runtime: Arc<RuntimeConfig>,
    pub config: OperationalConfig,
}

impl<TS: TransportSpec> Client<TS> {
    /// Creates a client builder whose terminal method is `build()`.
    pub fn new() -> AppBuilder<ClientRole, TS> {
        AppBuilder::new()
    }

    /// Returns the registered root URL tree for one protocol.
    pub fn root<P: Protocol<Wire = TS::Wire, Spec = TS> + 'static>(
        &self,
    ) -> Option<Arc<UrlRoot<P::Context, TS>>> {
        self.registry.url::<P>()
    }

    /// Returns the shared runtime mode.
    pub fn get_mode(self: &Arc<Self>) -> RunMode {
        self.runtime.mode()
    }

    /// Returns the shared runtime config store.
    pub fn config(self: &Arc<Self>) -> &crate::extensions::Params {
        self.runtime.config()
    }

    /// Returns the shared runtime statics store.
    pub fn statics(self: &Arc<Self>) -> &crate::extensions::Locals {
        self.runtime.statics()
    }

    /// Gets one typed config value from the shared runtime store.
    pub fn get_config<T: Clone + Send + Sync + 'static>(self: &Arc<Self>) -> Option<T> {
        self.runtime.get_config::<T>()
    }

    /// Gets one typed static value from the shared runtime store.
    pub fn get_static<T: Clone + Send + Sync + 'static>(self: &Arc<Self>, key: &str) -> Option<T> {
        self.runtime.get_static::<T>(key)
    }

    /// Gets one typed config value or its default.
    pub fn get_config_or_default<T: Clone + Default + Send + Sync + 'static>(
        self: &Arc<Self>,
    ) -> T {
        self.runtime.get_config::<T>().unwrap_or_default()
    }

    /// Gets one typed static value or its default.
    pub fn get_static_or_default<T: Clone + Default + Send + Sync + 'static>(
        self: &Arc<Self>,
        key: &str,
    ) -> T {
        self.runtime.get_static::<T>(key).unwrap_or_default()
    }

    /// Returns the configured outbound connection-pool size.
    pub fn get_worker(self: &Arc<Self>) -> usize {
        self.config.worker()
    }

    /// Returns the configured maximum connection lifetime setting.
    pub fn get_max_connection_time(self: &Arc<Self>) -> TimeoutSetting {
        self.config.max_connection_time()
    }

    /// Returns the configured maximum request processing time in seconds.
    pub fn get_max_frame_process_time(self: &Arc<Self>) -> usize {
        self.config.max_frame_process_time()
    }

    /// Opens one outbound wire to this client's configured target.
    pub async fn connect(self: &Arc<Self>) -> std::io::Result<TS::Wire> {
        TS::Outbound::connect(self.target.clone()).await
    }

    /// Runs protocol-side client handling on an existing wire.
    pub async fn run_wire(self: &Arc<Self>, wire: TS::Wire) {
        self.registry.run(self.runtime.clone(), wire).await;
    }

    /// Resolves an outbound path into a concrete endpoint node.
    pub async fn resolve<P: Protocol<Wire = TS::Wire, Spec = TS> + 'static>(
        self: &Arc<Self>,
        path: &str,
    ) -> Result<Arc<UrlNode<P::Context, TS>>, UrlError> {
        let Some(root) = self.root::<P>() else {
            return Err(UrlError::InvalidPath(path.to_string()));
        };

        root.walk_str(path)
            .await
            .ok_or_else(|| UrlError::InvalidPath(path.to_string()))
    }

    /// Resolves an outbound path with an explicit depth limit.
    pub async fn resolve_with_limit<P: Protocol<Wire = TS::Wire, Spec = TS> + 'static>(
        self: &Arc<Self>,
        path: &str,
        max_depth: u32,
    ) -> Result<Arc<UrlNode<P::Context, TS>>, UrlError> {
        let Some(root) = self.root::<P>() else {
            return Err(UrlError::InvalidPath(path.to_string()));
        };

        root.walk_str_with_limit(path, max_depth)
            .await
            .ok_or_else(|| UrlError::InvalidPath(path.to_string()))
    }

    /// Executes one outbound route by path and runs its middleware/final handler chain.
    pub async fn request<P: Protocol<Wire = TS::Wire, Spec = TS> + 'static>(
        self: &Arc<Self>,
        path: &str,
        ctx: P::Context,
    ) -> Result<P::Context, UrlError> {
        let endpoint = self.resolve::<P>(path).await?;
        Ok(endpoint.run(ctx).await)
    }

    /// Executes one outbound route by path with an explicit depth limit.
    pub async fn request_with_limit<P: Protocol<Wire = TS::Wire, Spec = TS> + 'static>(
        self: &Arc<Self>,
        path: &str,
        max_depth: u32,
        ctx: P::Context,
    ) -> Result<P::Context, UrlError> {
        let endpoint = self.resolve_with_limit::<P>(path, max_depth).await?;
        Ok(endpoint.run(ctx).await)
    }
}
