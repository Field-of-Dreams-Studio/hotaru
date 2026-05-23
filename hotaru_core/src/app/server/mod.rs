use akari::extensions::ParamsClone;
use core::panic;
use std::any::TypeId;

use std::sync::Arc;
use std::time::Duration;

use crate::executable::ExecutableBinding;
use crate::{debug_error, debug_log, debug_warn};

use crate::connection::{Inbound, TransportSpec};
use crate::protocol::{Protocol, RequestContext};
use crate::url::{PathPattern, UrlError, node::StepName};

pub use crate::executable::ProtocolRegistryBuilder;
pub use crate::app::registry::ProtocolRegistryKind;

// use super::middleware::AsyncMiddleware;
pub use super::common::builder::AppBuilder;
use super::common::builder::ServerRole;
use super::common::{OperationalConfig, RunMode, RuntimeConfig, TimeoutSetting};

// type Job = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Server runtime for inbound protocol traffic.
pub struct Server<TS: TransportSpec = crate::connection::tcp::TcpTransport> {
    pub registry: ProtocolRegistryKind<TS>, 
    pub binding: <TS::Inbound as Inbound>::BindTarget,
    pub inbound: tokio::sync::OnceCell<Arc<TS::Inbound>>,
    pub runtime: Arc<RuntimeConfig>,
    pub config: OperationalConfig,
}

impl<TS: TransportSpec> Server<TS> {
    /// Creates a server builder whose terminal method is `build()`.
    pub fn new() -> AppBuilder<ServerRole, TS> {
        AppBuilder::new()
    }

    // TODO : implement this method
    // pub fn get_protocol_address<P: Protocol>(&self) -> String {
    //     unimplemented!()
    // }

    pub fn get_mode(self: &Arc<Self>) -> RunMode {
        self.runtime.mode()
    }

    pub fn set_max_connection_time(&mut self, max_connection_time: TimeoutSetting) {
        self.config.set_max_connection_time(max_connection_time);
    }

    pub fn get_max_connection_time(self: &Arc<Self>) -> TimeoutSetting {
        self.config.max_connection_time()
    }

    pub fn get_max_frame_process_time(self: &Arc<Self>) -> usize {
        self.config.max_frame_process_time()
    }

    pub fn set_max_frame_process_time(&mut self, max_frame_process_time: usize) {
        self.config
            .set_max_frame_process_time(max_frame_process_time);
    }

    pub fn config(self: &Arc<Self>) -> &crate::extensions::Params {
        self.runtime.config()
    }

    pub fn statics(self: &Arc<Self>) -> &crate::extensions::Locals {
        self.runtime.statics()
    }

    pub fn get_config<T: Clone + Send + Sync + 'static>(self: &Arc<Self>) -> Option<T> {
        self.runtime.get_config::<T>()
    }

    pub fn get_static<T: Clone + Send + Sync + 'static>(self: &Arc<Self>, key: &str) -> Option<T> {
        self.runtime.get_static::<T>(key)
    }

    pub fn get_config_or_default<T: Clone + Default + Send + Sync + 'static>(
        self: &Arc<Self>,
    ) -> T {
        self.runtime.get_config::<T>().unwrap_or_default()
    }

    pub fn get_static_or_default<T: Clone + Default + Send + Sync + 'static>(
        self: &Arc<Self>,
        key: &str,
    ) -> T {
        self.runtime.get_static::<T>(key).unwrap_or_default()
    }

    /// Get the default protocol type (first registered protocol)
    /// TODO: What happen when empty - Should not return ()'s Type ID!
    pub fn default_protocol_type(self: &Arc<Self>) -> TypeId {
        // Return the first protocol's TypeId from registry
        self.registry
            .first_protocol_type_id()
            .unwrap_or_else(|| TypeId::of::<()>())
    }

    /// Register a literal URL — no pattern grammar; the string is split on
    /// `/` and each segment becomes a literal `PathPattern`. `name`
    /// identifies the access point for later lookup (used by `request_fn` /
    /// `run_fn` / `call_fn` and the trans-macro funnels).
    pub fn lit_url<P, T, N>(
        self: &Arc<Self>,
        url: T,
        name: N,
        mut executable: ExecutableBinding<P::Context>,
        config: ParamsClone,
    ) -> Result<(), UrlError>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        T: AsRef<str>,
        N: Into<String>,
    {
        // If no middleware is configured for this executable, set the protocol-level middlewares as default.
        if executable.has_no_middlewares() {
            executable.set_middlewares(self.registry.get_protocol_middlewares::<P>());
        }
        let url = url.as_ref();
        let path: Vec<PathPattern> = if url.is_empty() {
            Vec::new()
        } else {
            url.split('/').map(PathPattern::literal_path).collect()
        };
        self.registry.register::<P, _>(name, path, StepName::default(), executable, config)?;
        Ok(())
    }

    /// Register a URL using Hotaru pattern syntax (literals, `<name>`,
    /// `<type:name>`, `<regex>`, `*`, `**path`). `name` identifies the
    /// access point for later lookup.
    pub fn url<P, T, N>(
        self: &Arc<Self>,
        url: T,
        name: N,
        mut executable: ExecutableBinding<P::Context>,
        config: ParamsClone,
    ) -> Result<(), UrlError>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        T: AsRef<str>,
        N: Into<String>,
    {
        // If no middleware is configured for this executable, set the protocol-level middlewares as default.
        if executable.has_no_middlewares() {
            executable.set_middlewares(self.registry.get_protocol_middlewares::<P>());
        }
        let (path, step_names) = crate::url::parser::parse(url.as_ref())?;
        self.registry.register::<P, _>(name, path, step_names.into(), executable, config)?;
        Ok(())
    }

    // TODO: Implement register_from on Url or remove this method
    // pub fn reg_from<P: Protocol + 'static>(self: &Arc<Self>, segments: &[PathPattern]) -> Arc<Url<P::Context>> {
    //     match self.registry.reg_from::<P>(segments) {
    //         Ok(url) => url,
    //         Err(e) => {
    //             eprintln!("{}", e);
    //             dangling_url()
    //         }
    //     }
    // }

    /// Handle one accepted wire stream.
    pub fn handle_wire(self: Arc<Self>, conn: TS::Wire) {
        // Resolve Inherit to the protocol's own default before spawning.
        let timeout = match self.config.max_connection_time() {
            TimeoutSetting::Inherit => self.registry.default_connection_timeout(),
            TimeoutSetting::Disabled => None,
            TimeoutSetting::Fixed(d) => Some(d),
        };
        let app = self.clone();
        tokio::spawn(async move {
            match timeout {
                None => {
                    self.registry.serve(app.runtime.clone(), conn).await;
                }
                Some(duration) => {
                    tokio::select! {
                        _ = self.registry.serve(app.runtime.clone(), conn) => {},
                        _ = tokio::time::sleep(duration) => {
                            debug_warn!("⚠️ Connection timed out after {:?}", duration);
                        }
                    }
                }
            }
        });
    }

    /// Run the application with its own dedicated tokio runtime
    ///
    /// This method creates a new multi-threaded tokio runtime with the number of worker threads
    /// specified by the `worker` field (set via `AppBuilder::worker()`). Each Server instance
    /// runs with its own independent runtime and thread pool.
    ///
    /// Note: This can be called from within an async context. The worker thread configuration
    /// of any outer runtime does not affect the Server's internal worker thread count.
    ///
    /// Example:
    /// ```no_run
    /// use hotaru_core::app::server::Server;
    /// use hotaru_core::connection::tcp::TcpTransport;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let app = Server::<TcpTransport>::new()
    ///         .worker(4)  // Server will use 4 worker threads
    ///         .build();
    ///     app.run().await;
    /// }
    /// ```
    pub async fn run(self: Arc<Self>) {
        let worker_count = self.config.worker();
        let app = self.clone();

        println!("Starting Hotaru server");

        // Spawn a blocking task to create and run the runtime
        // This allows the runtime to be created from within an async context
        tokio::task::spawn_blocking(move || {
            // Create a new multi-threaded runtime with the specified worker threads
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(worker_count)
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime");

            // Run the actual server logic within this runtime
            runtime.block_on(app.run_app_loop());
        })
        .await
        .expect("Runtime task panicked");
    }

    /// Synthetically invoke a registered endpoint by name. Builds a fresh
    /// context, injects the request, runs the endpoint's chain, returns the
    /// response. No wire is opened — intended for tests and in-process
    /// simulation. Outer `UrlError` covers "name not found"; inner protocol
    /// error covers chain failures.
    pub async fn request_fn<P>(
        self: &Arc<Self>,
        name: &str,
        request: <P::Context as RequestContext>::Request,
    ) -> Result<
        Result<<P::Context as RequestContext>::Response, <P::Context as RequestContext>::Error>,
        UrlError,
    >
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
    {
        let entry = self
            .registry
            .entry::<P>()
            .ok_or(UrlError::ProtocolNotFound)?;
        let ap = entry
            .access_points
            .get(name)
            .ok_or_else(|| UrlError::InvalidPath(name.to_string()))?;
        let node = ap
            .resolve()
            .ok_or_else(|| UrlError::InvalidPath(name.to_string()))?;

        let mut ctx = P::Context::default();
        ctx.inject_request(request);

        let inner = match node.run(ctx).await {
            Ok(ctx) => Ok(ctx.into_response()),
            Err(e) => Err(e),
        };
        Ok(inner)
    }

    /// Returns the `TS::Inbound` instance, binding on first use.
    pub async fn ensure_inbound(&self) -> std::io::Result<&Arc<TS::Inbound>> {
        self.inbound
            .get_or_try_init(|| async {
                Ok(Arc::new(TS::Inbound::bind(self.binding.clone()).await?))
            })
            .await
    }

    /// Internal application loop - listens for and handles connections
    async fn run_app_loop(self: Arc<Self>) {
        let inbound = self
            .ensure_inbound()
            .await
            .unwrap_or_else(|_| panic!("Failed to bind inbound transport"))
            .clone();

        debug_log!("Inbound transport bound");

        // Create a signal handler for clean shutdown
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        // Handle Ctrl+C for clean shutdown
        tokio::spawn(async move {
            if let Ok(_) = tokio::signal::ctrl_c().await {
                debug_log!("Received shutdown signal");
                let _ = shutdown_tx.send(());
            }
        });

        loop {
            tokio::select! {
                accept_result = inbound.accept() => {
                    match accept_result {
                        Ok(conn) => {
                            debug_log!("Accepted inbound wire");
                            Arc::clone(&self).handle_wire(conn);
                        }
                        Err(_e) => {
                            if self.get_mode() == RunMode::Build{
                                debug_error!("Failed to accept connection: {_e}");
                            }
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    debug_log!("Shutting down server...");
                    break;
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
        debug_log!("Server shutdown complete");
    }
}
