use core::panic;
use std::any::TypeId; 
use akari::extensions::ParamsClone;
use tokio::net::{TcpListener, TcpStream};

use std::sync::Arc;
use std::time::Duration;

use crate::executable::ExecutableBinding; 
use crate::{debug_log, debug_error, debug_warn};

use crate::url::{UrlError}; 
use crate::connection::{Protocol, Accepter, TransportSpec};

pub mod registry;

pub use registry::ProtocolRegistryKind; 
pub use crate::executable::ProtocolRegistryBuilder;

// use super::middleware::AsyncMiddleware;
pub use super::common::builder::AppBuilder;
use super::common::{OperationalConfig, RunMode, RuntimeConfig};
use super::common::builder::ServerRole;

// type Job = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Server struct modified to store binding address instead of TcpListener
pub struct Server<TS: TransportSpec = crate::connection::tcp::TcpTransport> {
    pub handler: ProtocolRegistryKind<TS>, // Changed from listener to binding_address
    pub accepter: TS::Accepter,
    pub runtime: Arc<RuntimeConfig>,
    pub server: OperationalConfig,
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

    pub fn set_max_connection_time(&mut self, max_connection_time: usize) {
        self.server.set_max_connection_time(max_connection_time);
    }

    pub fn get_max_connection_time(self: &Arc<Self>) -> usize {
        self.server.max_connection_time()
    } 

    pub fn get_max_frame_process_time(self: &Arc<Self>) -> usize {
        self.server.max_frame_process_time()
    } 

    pub fn set_max_frame_process_time(&mut self, max_frame_process_time: usize) {
        self.server.set_max_frame_process_time(max_frame_process_time);
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
        self.handler
            .first_protocol_type_id()
            .unwrap_or_else(|| TypeId::of::<()>())
    } 

    /// This function add a new url to the app. It will be added to the root url
    /// # Arguments
    /// * `url` - The url to add. It should be a string.
    pub fn lit_url<P: Protocol<Wire = TS::Wire, Spec = TS> + 'static, T: Into<String>>(
        self: &Arc<Self>,
        url: T,
        mut executable: ExecutableBinding<P::Context>, 
        config: ParamsClone,
    ) -> Result<(), UrlError> { 
        // If no middleware is configured for this executable, set the protocol-level middlewares as default. 
        if executable.has_no_middlewares() {
            executable.set_middlewares(self.handler.get_protocol_middlewares::<P>()); 
        }
        self.handler.lit_url::<P, _>(url, executable, config)?; 
        Ok(()) 
    } 

    /// Regiter a URL by using Hotaru Pattern 
    pub fn url<P: Protocol<Wire = TS::Wire, Spec = TS> + 'static, T: Into<String>>(
        self: &Arc<Self>,
        url: T,
        mut executable: ExecutableBinding<P::Context>, 
        config: ParamsClone,
    ) -> Result<(), UrlError> { 
        // If no middleware is configured for this executable, set the protocol-level middlewares as default. 
        if executable.has_no_middlewares() {
            executable.set_middlewares(self.handler.get_protocol_middlewares::<P>()); 
        }
        self.handler.sub_url::<P, _>(url, executable, config)?; 
        Ok(()) 
    }

    // TODO: Implement register_from on Url or remove this method
    // pub fn reg_from<P: Protocol + 'static>(self: &Arc<Self>, segments: &[PathPattern]) -> Arc<Url<P::Context>> {
    //     match self.handler.reg_from::<P>(segments) {
    //         Ok(url) => url,
    //         Err(e) => {
    //             eprintln!("{}", e);
    //             dangling_url()
    //         }
    //     }
    // }

    /// Handle a single connection
    pub fn handle_connection(self: Arc<Self>, stream: TcpStream){
        let duration = Duration::from_secs(self.server.max_connection_time() as u64);
        let app = self.clone();
        tokio::spawn(async move {
            match self.accepter.upgrade(stream).await {
                Ok(conn) => {
                    tokio::select! {
                        _ = self.handler.run(app.runtime.clone(), conn) => {},
                        _ = tokio::time::sleep(duration) => {
                            debug_warn!("⚠️ Connection timed out after {:?}", duration);
                        }
                    }
                }
                Err(e) => {
                    debug_error!("Failed to upgrade accepted TCP connection: {e}");
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
        let worker_count = self.server.worker();
        let app = self.clone(); 

        println!("Starting Hotaru server on {}", self.server.binding_address()); 

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

    /// Internal application loop - listens for and handles connections
    async fn run_app_loop(self: Arc<Self>) {
        // Create TcpListener only when run() is called, within the tokio runtime
        // This function should directly panic because this is during the stage where APP is getting initialized
        // This error is unwindable
        let listener = TcpListener::bind(self.server.binding_address()).await.unwrap_or_else(|_| panic!("Failed to bind to address"));

        debug_log!(
            "Connection established on {}",
            match listener.local_addr() { 
                Ok(addr) => addr.to_string(),
                Err(e) => "error".to_string() + &e.to_string(),
            } 
        );

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
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, addr)) => {
                            debug_log!("Accepted connection from {addr}");
                            Arc::clone(&self).handle_connection(stream);
                        }
                        Err(e) => {
                            if self.get_mode() == RunMode::Build{
                                debug_error!("Failed to accept connection: {e}");
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
