use std::{marker::PhantomData, sync::Arc};

use crate::{
    app::{
        client::{Client, ProtocolRegistryKind as ClientProtocolRegistryKind},
        server::{ProtocolRegistryKind as ServerProtocolRegistryKind, Server},
    },
    connection::{Protocol, TransportSpec},
    executable::{ProtocolEntryBuilder, ProtocolRegistryBuilder, registry::ProtocolEntryRegistry},
    extensions::{Locals, Params},
};

use super::{OperationalConfig, RunMode, RuntimeConfig, TimeoutSetting};

pub struct ServerRole;
pub struct ClientRole;

/// Shared runtime builder used as the base for server and client construction.
///
/// The role marker decides which terminal `build()` method is available:
/// `AppBuilder<ServerRole, TS>` builds a [`Server`], while
/// `AppBuilder<ClientRole, TS>` builds a [`Client`].
pub struct AppBuilder<R, TS: TransportSpec = crate::connection::tcp::TcpTransport> {
    binding_address: Option<String>,
    registry: Option<ProtocolEntryRegistry<TS>>,
    accepter: Option<TS::Accepter>,
    connector: Option<TS::Connector>,
    mode: Option<RunMode>,
    worker: Option<usize>,
    max_connection_time: Option<TimeoutSetting>,
    max_frame_process_time: Option<usize>,
    config: Params,
    statics: Locals,
    _role: PhantomData<R>,
}

impl<R, TS: TransportSpec> AppBuilder<R, TS> {
    pub fn new() -> Self {
        Self {
            binding_address: None,
            registry: None,
            accepter: None,
            connector: None,
            mode: None,
            worker: None,
            max_connection_time: None,
            max_frame_process_time: None,
            config: Params::new(),
            statics: Locals::new(),
            _role: PhantomData,
        }
    }

    pub fn binding<T: Into<String>>(mut self, binding: T) -> Self {
        self.binding_address = Some(binding.into());
        self
    }

    pub fn registry(mut self, protocol: ProtocolEntryRegistry<TS>) -> Self {
        self.registry = Some(protocol);
        self
    }

    pub fn accepter(mut self, accepter: TS::Accepter) -> Self {
        self.accepter = Some(accepter);
        self
    }

    pub fn connector(mut self, connector: TS::Connector) -> Self {
        self.connector = Some(connector);
        self
    }

    pub fn handle(mut self, protocol: ProtocolRegistryBuilder<TS>) -> Self {
        self.registry = Some(protocol.build());
        self
    }

    pub fn single_protocol<P: Protocol<Wire = TS::Wire, Spec = TS>>(
        mut self,
        builder: ProtocolEntryBuilder<P, TS>,
    ) -> Self {
        self.registry = Some(
            ProtocolRegistryBuilder::<TS>::new()
                .protocol(builder)
                .build(),
        );
        self
    }

    pub fn mode(mut self, mode: RunMode) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn worker(mut self, threads: usize) -> Self {
        self.worker = Some(threads);
        self
    }

    pub fn max_connection_time(mut self, max_connection_time: TimeoutSetting) -> Self {
        self.max_connection_time = Some(max_connection_time);
        self
    }

    pub fn max_frame_process_time(mut self, max_frame_process_time: usize) -> Self {
        self.max_frame_process_time = Some(max_frame_process_time);
        self
    }

    pub fn statics(mut self, statics: Locals) -> Self {
        self.statics = statics;
        self
    }

    pub fn set_statics<K: Into<String>, V: Send + Sync + 'static>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.statics.set(key, value);
        self
    }

    pub fn config(mut self, config: Params) -> Self {
        self.config = config;
        self
    }

    pub fn set_config<V: Send + Sync + 'static>(mut self, value: V) -> Self {
        self.config.set(value);
        self
    }
}

impl<TS: TransportSpec> AppBuilder<ServerRole, TS> {
    /// Builds a server runtime from the configured server-side builder state.
    pub fn build(self) -> Arc<super::super::server::Server<TS>> {
        let handler = self
            .registry
            .map(ServerProtocolRegistryKind::from)
            .expect("AppBuilder::registry(...) must be set for App<TS>");
        let accepter = self
            .accepter
            .or_else(TS::default_accepter)
            .expect("AppBuilder::accepter(...) must be set for Server<TS>");

        let binding_address = self
            .binding_address
            .unwrap_or_else(|| String::from("127.0.0.1:3003"));
        let mode = self.mode.unwrap_or(RunMode::Development);
        let worker = self.worker.unwrap_or_else(num_cpus);
        let max_connection_time = self
            .max_connection_time
            .unwrap_or(TimeoutSetting::Inherit);
        let max_frame_process_time = self.max_frame_process_time.unwrap_or(5);
        let runtime = RuntimeConfig::from_parts(mode, self.config, self.statics);
        let server = OperationalConfig::from_server_parts(
            binding_address,
            worker,
            max_connection_time,
            max_frame_process_time,
        );
        let runtime = Arc::new(runtime);

        let app = Arc::new(Server {
            handler,
            accepter,
            runtime,
            server,
        });

        app
    }
}

impl<TS: TransportSpec> AppBuilder<ClientRole, TS> {
    /// Builds a client runtime from the configured client-side builder state.
    pub fn build(self) -> Arc<Client<TS>> {
        let session = self
            .registry
            .map(ClientProtocolRegistryKind::from)
            .expect("AppBuilder::registry(...) must be set for Client<TS>");
        let connector = self
            .connector
            .or_else(TS::default_connector)
            .expect("AppBuilder::connector(...) must be set for Client<TS>");

        let mode = self.mode.unwrap_or(RunMode::Development);
        let connect_timeout = self
            .max_connection_time
            .unwrap_or(TimeoutSetting::Seconds(30));
        let request_timeout = self
            .max_frame_process_time
            .map(|n| TimeoutSetting::Seconds(n))
            .unwrap_or(TimeoutSetting::Seconds(30));
        let runtime = Arc::new(RuntimeConfig::from_parts(mode, self.config, self.statics));
        let client = OperationalConfig::from_client_parts(connect_timeout, request_timeout);

        Arc::new(Client {
            session,
            connector,
            runtime,
            client,
        })
    }
}

// Helper function for determining CPU count
fn num_cpus() -> usize {
    match std::thread::available_parallelism() {
        Ok(n) => n.get(),
        Err(_) => 1, // Fallback if we can't determine
    }
}
