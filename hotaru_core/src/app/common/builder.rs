use alloc::sync::Arc;
use core::marker::PhantomData;

use crate::{
    app::{
        client::Client,
        registry::ProtocolRegistryKind,
        server::Server,
    },
    connection::{Inbound, Outbound, TransportSpec},
    executable::{ProtocolEntryBuilder, ProtocolRegistryBuilder, registry::ProtocolEntryRegistry},
    extensions::{Locals, Params},
    protocol::Protocol,
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
    registry: Option<ProtocolEntryRegistry<TS>>,
    binding: Option<<TS::Inbound as Inbound>::BindTarget>,
    target: Option<<TS::Outbound as Outbound>::ConnectTarget>,
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
            registry: None,
            binding: None,
            target: None,
            mode: None,
            worker: None,
            max_connection_time: None,
            max_frame_process_time: None,
            config: Params::new(),
            statics: Locals::new(),
            _role: PhantomData,
        }
    }

    pub fn binding<T: Into<String>>(self, binding: T) -> Self
    where
        <TS::Inbound as Inbound>::BindTarget: From<String>,
    {
        self.with_binding(binding.into().into())
    }

    pub fn registry(mut self, protocol: ProtocolEntryRegistry<TS>) -> Self {
        self.registry = Some(protocol);
        self
    }

    pub fn with_binding(mut self, binding: <TS::Inbound as Inbound>::BindTarget) -> Self {
        self.binding = Some(binding);
        self
    }

    pub fn target(mut self, target: <TS::Outbound as Outbound>::ConnectTarget) -> Self {
        self.target = Some(target);
        self
    }

    pub fn handle(mut self, protocol: ProtocolRegistryBuilder<TS>) -> Self {
        self.registry = Some(protocol.build());
        self
    }

    pub fn single_protocol<P: Protocol<Wire = TS::Wire, TS = TS>>(
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
    /// 
    /// Panics - server runtimes require a protocol registry, so this must be set
    /// via the builder methods before calling `build()`. 
    pub fn build(self) -> Arc<super::super::server::Server<TS>> {
        let registry = self
            .registry
            .map(ProtocolRegistryKind::from)
            .expect("AppBuilder::registry(...) must be set for App<TS>");
        let binding = self
            .binding
            .or_else(TS::default_inbound)
            .expect("AppBuilder::binding(...) must be set for Server<TS>");

        let mode = self.mode.unwrap_or(RunMode::Development);
        let worker = self.worker.unwrap_or_else(num_cpus);
        let max_connection_time = self.max_connection_time.unwrap_or(TimeoutSetting::Inherit);
        let max_frame_process_time = self.max_frame_process_time.unwrap_or(5);
        let runtime = RuntimeConfig::from_parts(mode, self.config, self.statics);
        let config = OperationalConfig::from_server_parts(
            worker,
            max_connection_time,
            max_frame_process_time,
        );
        let runtime = Arc::new(runtime);

        let app = Arc::new(Server {
            registry,
            binding,
            inbound: Default::default(),
            runtime,
            config,
            _rt: PhantomData,
        });

        app
    }
}

impl<TS: TransportSpec> AppBuilder<ClientRole, TS> {
    /// Builds a client runtime from the configured client-side builder state.
    /// 
    /// Panics - client runtimes require a target and protocol registry, so these must be set
    /// via the builder methods before calling `build()`. 
    pub fn build(self) -> Arc<Client<TS>> {
        let registry = self
            .registry
            .map(ProtocolRegistryKind::from)
            .expect("AppBuilder::registry(...) must be set for Client<TS>");
        let target = self
            .target
            .or_else(TS::default_outbound)
            .expect("AppBuilder::target(...) must be set for Client<TS>");

        let mode = self.mode.unwrap_or(RunMode::Development);
        let connect_timeout = self
            .max_connection_time
            .unwrap_or(TimeoutSetting::Seconds(30));
        let request_timeout = self
            .max_frame_process_time
            .map(|n| TimeoutSetting::Seconds(n))
            .unwrap_or(TimeoutSetting::Seconds(30));
        let runtime = Arc::new(RuntimeConfig::from_parts(mode, self.config, self.statics));
        let config = OperationalConfig::from_client_parts(connect_timeout, request_timeout);

        Arc::new(Client {
            registry,
            target,
            outbound: Default::default(),
            runtime,
            config,
            _rt: PhantomData,
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
