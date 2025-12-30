use core::panic;
use std::any::TypeId;
// use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};

// use starberry_lib::random_string;
// use std::future::Future;
// use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use crate::{debug_log, debug_error, debug_warn};

use crate::url::{Url, dangling_url}; 
use crate::app::protocol::{ProtocolHandlerBuilder, ProtocolRegistryBuilder};
use crate::connection::TcpConnectionStream;
use crate::connection::Protocol;

use crate::extensions::{Params, Locals}; 
use crate::http::context::HttpReqCtx;

// use super::middleware::AsyncMiddleware;
use super::protocol::ProtocolRegistryKind;

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
pub struct App {
    pub binding_address: String,
    pub handler: ProtocolRegistryKind, // Changed from listener to binding_address
    pub mode: RunMode,
    pub worker: usize, // Number of worker threads for the app's tokio runtime
    pub max_connection_time: usize,
    pub max_frame_process_time: usize,
    pub config: Params,
    pub statics: Locals,
}

/// Builder for App
pub struct AppBuilder {
    binding_address: Option<String>,
    handler: Option<ProtocolRegistryKind>,
    mode: Option<RunMode>,
    worker: Option<usize>,
    max_connection_time: Option<usize>, 
    max_frame_process_time: Option<usize>, 
    config: Params, 
    statics: Locals, 
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            binding_address: None,
            handler: None,
            mode: None,
            worker: None,
            max_connection_time: None, 
            max_frame_process_time: None, 
            config: Params::new(),  
            statics: Locals::new(), 
        }
    }

    /// Set the binding address for the application 
    pub fn binding<T: Into<String>>(mut self, binding: T) -> Self {
        self.binding_address = Some(binding.into());
        self
    }

    /// Set the handler for the application
    pub fn handler(mut self, protocol: ProtocolRegistryKind) -> Self {
        self.handler = Some(protocol);
        self
    } 

    /// Set the handler for the application using a ProtocolRegistryBuilder 
    pub fn handle(mut self, protocol: ProtocolRegistryBuilder) -> Self { 
        self.handler = Some(protocol.build());
        self 
    } 

    /// Set the handler for the application using a ProtocolHandlerBuilder 
    /// This works for a single protocol appication 
    pub fn single_protocol<P: Protocol>(mut self, builder: ProtocolHandlerBuilder<P>) -> Self {
        self.handler = Some(
            ProtocolRegistryBuilder::new()
                .protocol(builder)
                .build(),
        ); 
        self 
    } 

    /// Set the run mode for the application 
    pub fn mode(mut self, mode: RunMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Set the number of worker threads for the application's tokio runtime
    ///
    /// This controls how many threads the App's internal runtime will use for handling
    /// async tasks and connections. The default is the number of CPU cores.
    ///
    /// Note: This setting is independent of any outer tokio runtime. When `run()` is called,
    /// the App creates its own runtime with this many worker threads.
    pub fn worker(mut self, threads: usize) -> Self {
        self.worker = Some(threads);
        self
    }

    /// Set the maximum connection time for the application 
    pub fn max_connection_time(mut self, max_connection_time: usize) -> Self {
        self.max_connection_time = Some(max_connection_time);
        self
    } 

    /// Set the maxium process time for a frame 
    pub fn max_frame_process_time(mut self, max_frame_process_time: usize) -> Self {
        self.max_frame_process_time = Some(max_frame_process_time);
        self
    } 

    /// Set the FULL LOCAL HASHMAP for the application 
    pub fn statics(mut self, statics: Locals) -> Self {
        self.statics = statics; 
        self
    } 

    /// Set a single static value in the statics map 
    pub fn set_statics<K: Into<String>, V: Send + Sync + 'static>(mut self, key: K, value: V) -> Self {
        self.statics.set(key, value);
        self 
    } 

    /// Set the FULL PARAMS HASHMAP for the application 
    pub fn config(mut self, config: Params) -> Self {
        self.config = config; 
        self
    } 

    /// Set a single config value in the config map 
    pub fn set_config<V: Send + Sync + 'static>(mut self, value: V) -> Self { 
        self.config.set(value);
        self 
    }

    /// Build method: create the `App`, storing binding address without creating a TcpListener
    pub fn build(self) -> Arc<App> {
        let handler = match self.handler {
            Some(root_url) => root_url,
            None => ProtocolRegistryBuilder::new()
                .protocol(ProtocolHandlerBuilder::new(
                    crate::http::traits::Http1Protocol::server(
                        crate::http::safety::HttpSafety::default()
                    )
                ))
                .build(),
        };

        let binding_address = self
            .binding_address
            .unwrap_or_else(|| String::from("127.0.0.1:3003"));
        let mode = self.mode.unwrap_or_else(|| RunMode::Development);
        let worker = self.worker.unwrap_or_else(|| num_cpus());
        let max_connection_time = self.max_connection_time.unwrap_or_else(|| 30);  
        let max_frame_process_time = self.max_frame_process_time.unwrap_or_else(|| 5); 

        let app = Arc::new(App {
            handler,
            binding_address,
            mode,
            worker,
            max_connection_time,
            max_frame_process_time,
            config: self.config,
            statics: self.statics,
        });

        app.handler.attach_app(app.clone());

        app
    }
}

impl App {
    pub fn new() -> AppBuilder {
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

    /// Get the default protocol type (first registered protocol)
    pub fn default_protocol_type(self: &Arc<Self>) -> TypeId {
        // Return the first protocol's TypeId from registry
        self.handler.first_protocol_type_id().unwrap_or_else(|| TypeId::of::<HttpReqCtx>())
    } 

    /// This function add a new url to the app. It will be added to the root url
    /// # Arguments
    /// * `url` - The url to add. It should be a string.
    pub fn lit_url<P: Protocol + 'static, T: Into<String>>(
        self: &Arc<Self>,
        url: T,
    ) -> Arc<Url<P::Context>> {
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
    pub fn url<P: Protocol + 'static, A: AsRef<str>>(
        self: &Arc<Self>,
        url: A,
    ) -> Arc<Url<P::Context>> { 
        match self.handler.sub_url::<P, _>(url){
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
    pub fn handle_connection(self: Arc<Self>, stream: TcpStream) {
        let duration = Duration::from_secs(self.max_connection_time as u64);
        let app = self.clone();
        // 1) spawn the actual connection job
        // let handle = tokio::spawn(async move {
        //     self.handler.run(app, Connection::Tcp(stream)).await;
        // });
        // 2) in parallel, sleep then abort
        tokio::spawn(async move {
            tokio::select! { 
                _ = self.handler.run(app, TcpConnectionStream::Tcp(stream)) => {}, 
                _ = tokio::time::sleep(duration) => {
                    // Timed out: forcefully close
                    debug_warn!("⚠️ Connection timed out after {:?}", duration);
                    // Note: dropping the reader/writer will close the socket
                } 
            }  
            // tokio::time::sleep(duration).await;
            // if !handle.is_finished() {
            //     handle.abort();
            //     eprintln!("Connection timed out after {:?}", duration);
            // }
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
            listener.local_addr().unwrap_or_else(|_| "unknown")
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

// Helper function for determining CPU count
fn num_cpus() -> usize {
    match std::thread::available_parallelism() {
        Ok(n) => n.get(),
        Err(_) => 1, // Fallback if we can't determine
    }
}
