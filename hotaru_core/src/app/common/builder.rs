use crate::prelude::Arc;
#[cfg(not(feature = "std"))]
use crate::prelude::*;
use core::marker::PhantomData;

use crate::app::blueprint::{Blueprint, BlueprintError, ConfiguredBlueprint};
use crate::{
    app::{
        instance::{
            App,
            client::Client,
            server::Server,
            target::{InboundOnly, InboundState, OutboundOnly, OutboundState},
        },
        registry::ProtocolRegistryKind,
        runtime::RuntimeSpec,
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
/// `AppBuilder<ServerRole, TS, Rt>` builds a [`Server`], while
/// `AppBuilder<ClientRole, TS, Rt>` builds a [`Client`].
pub struct AppBuilder<R, TS: TransportSpec, Rt: RuntimeSpec> {
    registry: Option<ProtocolEntryRegistry<TS>>,
    binding: Option<<TS::Inbound as Inbound>::BindTarget>,
    target: Option<<TS::Outbound as Outbound>::ConnectTarget>,
    mode: Option<RunMode>,
    worker: Option<usize>,
    max_connection_time: Option<TimeoutSetting>,
    /// Server-side: budget for processing one request/frame inside a live
    /// connection. Consumed by [`ServerRole::build`]; ignored by clients.
    max_frame_process_timeout: Option<TimeoutSetting>,
    /// Client-side: end-to-end request timeout. Consumed by
    /// [`ClientRole::build`]; ignored by servers.
    request_timeout: Option<TimeoutSetting>,
    config: Params,
    statics: Locals,
    _role: PhantomData<R>,
    _rt: PhantomData<fn() -> Rt>,
}

impl<R, TS: TransportSpec, Rt: RuntimeSpec> AppBuilder<R, TS, Rt> {
    pub fn new() -> Self {
        Self {
            registry: None,
            binding: None,
            target: None,
            mode: None,
            worker: None,
            max_connection_time: None,
            max_frame_process_timeout: None,
            request_timeout: None,
            config: Params::new(),
            statics: Locals::new(),
            _role: PhantomData,
            _rt: PhantomData,
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

    /// Server-side per-frame processing budget. Ignored by `ClientRole::build`.
    pub fn max_frame_process_timeout(mut self, timeout: TimeoutSetting) -> Self {
        self.max_frame_process_timeout = Some(timeout);
        self
    }

    /// Client-side end-to-end request timeout. Ignored by `ServerRole::build`.
    pub fn request_timeout(mut self, timeout: TimeoutSetting) -> Self {
        self.request_timeout = Some(timeout);
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

    // -----------------------------------------------------------------------
    // Read-only accessors.
    //
    // Setters share the field name (`.mode(...)`, `.worker(...)`, …) so
    // getters take the `get_` prefix. Same convention as `App::get_*`.
    // Options are returned as-is: `None` means "no explicit value yet; the
    // role's `build()` will fall back to its own default."
    // -----------------------------------------------------------------------

    pub fn get_registry(&self) -> Option<&ProtocolEntryRegistry<TS>> {
        self.registry.as_ref()
    }

    pub fn get_binding(&self) -> Option<&<TS::Inbound as Inbound>::BindTarget> {
        self.binding.as_ref()
    }

    pub fn get_target(&self) -> Option<&<TS::Outbound as Outbound>::ConnectTarget> {
        self.target.as_ref()
    }

    pub fn get_mode(&self) -> Option<&RunMode> {
        self.mode.as_ref()
    }

    pub fn get_worker(&self) -> Option<usize> {
        self.worker
    }

    pub fn get_max_connection_time(&self) -> Option<TimeoutSetting> {
        self.max_connection_time
    }

    pub fn get_max_frame_process_timeout(&self) -> Option<TimeoutSetting> {
        self.max_frame_process_timeout
    }

    pub fn get_request_timeout(&self) -> Option<TimeoutSetting> {
        self.request_timeout
    }

    pub fn get_config(&self) -> &Params {
        &self.config
    }

    pub fn get_statics(&self) -> &Locals {
        &self.statics
    }
}

impl<TS: TransportSpec, Rt: RuntimeSpec> AppBuilder<ServerRole, TS, Rt> {
    /// Builds a server runtime from the configured server-side builder state.
    ///
    /// Panics - server runtimes require a protocol registry, so this must be set
    /// via the builder methods before calling `build()`.
    pub fn build(self) -> Arc<Server<TS, Rt>> {
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
        let max_frame_process_timeout = self
            .max_frame_process_timeout
            .unwrap_or(TimeoutSetting::Seconds(5));
        let runtime = RuntimeConfig::from_parts(mode, self.config, self.statics);
        let config = OperationalConfig::from_server_parts(
            worker,
            max_connection_time,
            max_frame_process_timeout,
        );
        let runtime = Arc::new(runtime);

        let app = Arc::new(App::<TS, Rt, InboundOnly> {
            registry,
            inbound_state: InboundState {
                binding,
                inbound: Default::default(),
            },
            outbound_state: (),
            runtime,
            config,
            _rt: PhantomData,
            _target: PhantomData,
        });

        app
    }

    /// Materializes the Blueprint, then installs or left-biased-merges it.
    pub fn apply(mut self, blueprint: &Blueprint<TS, InboundOnly>) -> Result<Self, BlueprintError> {
        let materialized_blueprint_registry = blueprint.materialize_registry()?;
        match self.registry.as_mut() {
            Some(existing) => existing.combine(materialized_blueprint_registry),
            None => self.registry = Some(materialized_blueprint_registry),
        }
        Ok(self)
    }

    /// `apply` plus defaults written only into unset builder fields.
    pub fn apply_configured(
        mut self,
        configured: &ConfiguredBlueprint<TS, InboundOnly>,
    ) -> Result<Self, BlueprintError> {
        self = self.apply(configured.blueprint())?;
        if self.mode.is_none() {
            self.mode = configured.mode().cloned();
        }
        if let Some(operational) = configured.operational() {
            if self.worker.is_none() {
                self.worker = Some(operational.worker());
            }
            if self.max_connection_time.is_none() {
                self.max_connection_time = Some(operational.max_connection_time());
            }
            if self.max_frame_process_timeout.is_none() {
                self.max_frame_process_timeout = Some(operational.max_frame_process_timeout());
            }
        }
        Ok(self)
    }
}

impl<TS: TransportSpec, Rt: RuntimeSpec> AppBuilder<ClientRole, TS, Rt> {
    /// Builds a client runtime from the configured client-side builder state.
    ///
    /// Panics - client runtimes require a target and protocol registry, so these must be set
    /// via the builder methods before calling `build()`.
    pub fn build(self) -> Arc<Client<TS, Rt>> {
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
        let request_timeout = self.request_timeout.unwrap_or(TimeoutSetting::Seconds(5));
        let runtime = Arc::new(RuntimeConfig::from_parts(mode, self.config, self.statics));
        let config = OperationalConfig::from_client_parts(connect_timeout, request_timeout);

        Arc::new(App::<TS, Rt, OutboundOnly> {
            registry,
            inbound_state: (),
            outbound_state: OutboundState {
                target,
                outbound: Default::default(),
            },
            runtime,
            config,
            _rt: PhantomData,
            _target: PhantomData,
        })
    }

    /// Materializes the Blueprint, then installs or left-biased-merges it.
    pub fn apply(
        mut self,
        blueprint: &Blueprint<TS, OutboundOnly>,
    ) -> Result<Self, BlueprintError> {
        let materialized_blueprint_registry = blueprint.materialize_registry()?;
        match self.registry.as_mut() {
            Some(existing) => existing.combine(materialized_blueprint_registry),
            None => self.registry = Some(materialized_blueprint_registry),
        }
        Ok(self)
    }

    /// `apply` plus defaults written only into unset builder fields.
    pub fn apply_configured(
        mut self,
        configured: &ConfiguredBlueprint<TS, OutboundOnly>,
    ) -> Result<Self, BlueprintError> {
        self = self.apply(configured.blueprint())?;
        if self.mode.is_none() {
            self.mode = configured.mode().cloned();
        }
        if let Some(operational) = configured.operational() {
            if self.max_connection_time.is_none() {
                self.max_connection_time = Some(operational.connect_timeout());
            }
            if self.request_timeout.is_none() {
                self.request_timeout = Some(operational.request_timeout());
            }
        }
        Ok(self)
    }
}

// Helper function for determining CPU count. `std::thread::available_parallelism`
// has no core equivalent; embedded builds fall back to a single worker (the
// only sensible default under a single-executor runtime).
#[cfg(feature = "std")]
fn num_cpus() -> usize {
    match std::thread::available_parallelism() {
        Ok(n) => n.get(),
        Err(_) => 1,
    }
}

#[cfg(not(feature = "std"))]
fn num_cpus() -> usize {
    1
}
