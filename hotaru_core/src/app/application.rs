use core::panic;
use std::any::TypeId; 
use tokio::net::{TcpListener, TcpStream};

use std::sync::Arc;
use std::time::Duration;

use crate::{debug_log, debug_error, debug_warn};

use crate::url::{Url, dangling_url}; 
use crate::connection::{Protocol, Accepter, TransportSpec};

use crate::extensions::{Params, Locals}; 

// use super::middleware::AsyncMiddleware;
pub use super::builder::AppBuilder;
use super::handler::ProtocolRegistryKind;

/// RunMode enum to represent the mode of the application
/// Production: Production mode
/// Development: Test on developer's computer, showing the error message and some debug info. May contain sensitive info.
/// Beta: Beta mode, showing some debug info. May contain some sensitive info.
/// Build: Build mode. For testing starberry itself. It will print out any information possible. 
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunMode {
    Production,
    Development,
    Beta,
    Build,
}

// type Job = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// App struct modified to store binding address instead of TcpListener
pub struct App<TS: TransportSpec = crate::connection::tcp::TcpTransport> {
    pub binding_address: String,
    pub handler: ProtocolRegistryKind<TS>, // Changed from listener to binding_address
    pub accepter: TS::Accepter,
    pub mode: RunMode,
    pub worker: usize, // Number of worker threads for the app's tokio runtime
    pub max_connection_time: usize,
    pub max_frame_process_time: usize,
    pub config: Params,
    pub statics: Locals,
}

impl<TS: TransportSpec> App<TS> {
    pub fn new() -> AppBuilder<TS> {
        AppBuilder::new()
    }

    // TODO : implement this method 
    // pub fn get_protocol_address<P: Protocol>(&self) -> String {
    //     unimplemented!() 
    // } 

    pub fn set_mode(&mut self, mode: RunMode) {
        self.mode = mode;
    }

    pub fn get_mode(self: &Arc<Self>) -> RunMode {
        self.mode.clone()
    }

    pub fn set_max_connection_time(&mut self, max_connection_time: usize) {
        self.max_connection_time = max_connection_time;
    }

    pub fn get_max_connection_time(self: &Arc<Self>) -> usize {
        self.max_connection_time
    } 

    pub fn get_max_frame_process_time(self: &Arc<Self>) -> usize {
        self.max_frame_process_time
    } 

    pub fn set_max_frame_process_time(&mut self, max_frame_process_time: usize) {
        self.max_frame_process_time = max_frame_process_time;
    } 

    pub fn config(self: &Arc<Self>) -> &Params {
        &self.config 
    } 

    pub fn statics(self: &Arc<Self>) -> &Locals {
        &self.statics
    } 

    pub fn get_config<T: Clone + Send + Sync + 'static>(self: &Arc<Self>) -> Option<T> {
        self.config.get::<T>().cloned()
    }

    pub fn get_static<T: Clone + Send + Sync + 'static>(self: &Arc<Self>, key: &str) -> Option<T> {
        self.statics.get::<T>(key).cloned()
    }

    pub fn get_config_or_default<T: Clone + Default + Send + Sync + 'static>(
        self: &Arc<Self>,
    ) -> T {
        self.config.get::<T>().cloned().unwrap_or_default()
    }

    pub fn get_static_or_default<T: Clone + Default + Send + Sync + 'static>(
        self: &Arc<Self>,
        key: &str,
    ) -> T {
        self.statics.get::<T>(key).cloned().unwrap_or_default()
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
    pub fn lit_url<P: Protocol<Wire = TS::Wire> + 'static, T: Into<String>>(
        self: &Arc<Self>,
        url: T,
    ) -> Arc<Url<P::Context, TS>> {
        match self.handler.lit_url::<P, _>(url) {
            Ok(url) => {
                url.set_middlewares(self.handler.get_protocol_middlewares::<P>()); 
                url 
            },
            Err(e) => {
                debug_error!("{}", e);
                dangling_url()
            } 
        }
    } 

    /// Regiter a URL by using Hotaru Pattern 
    pub fn url<P: Protocol<Wire = TS::Wire> + 'static, AR: AsRef<str>>(
        self: &Arc<Self>,
        url: AR,
    ) -> Arc<Url<P::Context, TS>> { 
        match self.handler.sub_url::<P, _>(url){
            Ok(url) => {
                url.set_middlewares(self.handler.get_protocol_middlewares::<P>()); 
                url 
            },
            Err(_e) => {
                debug_error!("{}", _e);
                dangling_url()
            }
        }
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
        let duration = Duration::from_secs(self.max_connection_time as u64);
        let app = self.clone();
        tokio::spawn(async move {
            match self.accepter.upgrade(stream).await {
                Ok(conn) => {
                    tokio::select! {
                        _ = self.handler.run(app, conn) => {},
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
    /// specified by the `worker` field (set via `AppBuilder::worker()`). Each App instance
    /// runs with its own independent runtime and thread pool.
    ///
    /// Note: This can be called from within an async context. The worker thread configuration
    /// of any outer runtime does not affect the App's internal worker thread count.
    ///
    /// Example:
    /// ```no_run
    /// use hotaru_core::app::App;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let app = App::new()
    ///         .worker(4)  // App will use 4 worker threads
    ///         .build();
    ///     app.run().await;
    /// }
    /// ```
    pub async fn run(self: Arc<Self>) {
        let worker_count = self.worker;
        let app = self.clone(); 

        println!("Starting Hotaru server on {}", self.binding_address); 

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
        let listener = TcpListener::bind(&self.binding_address).await.unwrap_or_else(|_| panic!("Failed to bind to address"));

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
