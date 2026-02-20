use std::sync::Arc;

use crate::{
    connection::{Protocol, TransportSpec},
    extensions::{Locals, Params},
};

use super::{
    application::{App, RunMode},
    handler::{ProtocolHandlerBuilder, ProtocolRegistryBuilder, ProtocolRegistryKind},
};

/// Builder for App
pub struct AppBuilder<TS: TransportSpec = crate::connection::tcp::TcpTransport> {
    binding_address: Option<String>,
    handler: Option<ProtocolRegistryKind<TS>>,
    accepter: Option<TS::Accepter>,
    mode: Option<RunMode>,
    worker: Option<usize>,
    max_connection_time: Option<usize>,
    max_frame_process_time: Option<usize>,
    config: Params,
    statics: Locals,
}

impl<TS: TransportSpec> AppBuilder<TS> {
    pub fn new() -> Self {
        Self {
            binding_address: None,
            handler: None,
            accepter: None,
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
    pub fn handler(mut self, protocol: ProtocolRegistryKind<TS>) -> Self {
        self.handler = Some(protocol);
        self
    }

    /// Set the accepter for inbound connection upgrade.
    pub fn accepter(mut self, accepter: TS::Accepter) -> Self {
        self.accepter = Some(accepter);
        self
    }

    /// Set the handler for the application using a ProtocolRegistryBuilder
    pub fn handle(mut self, protocol: ProtocolRegistryBuilder<TS>) -> Self {
        self.handler = Some(protocol.build());
        self
    }

    /// Set the handler for the application using a ProtocolHandlerBuilder
    /// This works for a single protocol appication
    pub fn single_protocol<P: Protocol<Wire = TS::Wire>>(
        mut self,
        builder: ProtocolHandlerBuilder<P, TS>,
    ) -> Self {
        self.handler = Some(ProtocolRegistryBuilder::<TS>::new().protocol(builder).build());
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
    pub fn set_statics<K: Into<String>, V: Send + Sync + 'static>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
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
    pub fn build(self) -> Arc<App<TS>> {
        let handler = self
            .handler
            .expect("AppBuilder::handler(...) must be set for App<TS>");
        let accepter = self
            .accepter
            .or_else(TS::default_accepter)
            .expect("AppBuilder::accepter(...) must be set for App<TS>");

        let binding_address = self
            .binding_address
            .unwrap_or_else(|| String::from("127.0.0.1:3003"));
        let mode = self.mode.unwrap_or(RunMode::Development);
        let worker = self.worker.unwrap_or_else(num_cpus);
        let max_connection_time = self.max_connection_time.unwrap_or(30);
        let max_frame_process_time = self.max_frame_process_time.unwrap_or(5);

        let app = Arc::new(App {
            handler,
            accepter,
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

// Helper function for determining CPU count
fn num_cpus() -> usize {
    match std::thread::available_parallelism() {
        Ok(n) => n.get(),
        Err(_) => 1, // Fallback if we can't determine
    }
}
