//! Unified application storage shared by inbound, outbound, and dual-role apps.

use core::marker::PhantomData;
use core::time::Duration;

use crate::app::blueprint::{Blueprint, BlueprintError};
use crate::app::common::{AppInUse, OperationalConfig, RunMode, RuntimeConfig, TimeoutSetting};
use crate::app::registry::ProtocolRegistryKind;
use crate::app::runtime::RuntimeSpec;
use crate::connection::TransportSpec;
use crate::executable::def::{AccessPointDef, BindError, FinalHandlerDef};
use crate::prelude::Arc;
use crate::protocol::Protocol;
use crate::{debug_log, debug_warn};

use super::{flavour::Accepts, target::AppTarget};

/// Unified application state.
///
/// `T` selects the concrete inbound and outbound side-state types. For
/// example, `InboundOnly` stores an `InboundState` and `()`, while `Both`
/// stores both concrete side states.
pub struct App<TS: TransportSpec, Rt: RuntimeSpec, T: AppTarget> {
    pub registry: ProtocolRegistryKind<TS>,
    pub inbound_state: T::Inbound<TS, Rt>,
    pub outbound_state: T::Outbound<TS, Rt>,
    pub runtime: Arc<RuntimeConfig>,
    pub config: OperationalConfig,
    pub(crate) _rt: PhantomData<fn() -> Rt>,
    pub(crate) _target: PhantomData<fn() -> T>,
}

/// Role-independent application behavior.
///
/// These methods depend only on storage shared by every target. Inbound- and
/// outbound-specific behavior lives in their respective modules.
impl<TS: TransportSpec, Rt: RuntimeSpec, T: AppTarget> App<TS, Rt, T> {
    /// Returns the shared runtime mode.
    pub fn get_mode(self: &Arc<Self>) -> RunMode {
        self.runtime.mode()
    }

    /// Returns the configured maximum connection lifetime setting.
    pub fn get_max_connection_time(self: &Arc<Self>) -> TimeoutSetting {
        self.config.max_connection_time()
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
    pub fn get_config<V: Clone + Send + Sync + 'static>(self: &Arc<Self>) -> Option<V> {
        self.runtime.get_config::<V>()
    }

    /// Gets one typed static value from the shared runtime store.
    pub fn get_static<V: Clone + Send + Sync + 'static>(self: &Arc<Self>, key: &str) -> Option<V> {
        self.runtime.get_static::<V>(key)
    }

    /// Gets one typed config value or its default.
    pub fn get_config_or_default<V: Clone + Default + Send + Sync + 'static>(
        self: &Arc<Self>,
    ) -> V {
        self.runtime.get_config::<V>().unwrap_or_default()
    }

    /// Gets one typed static value or its default.
    pub fn get_static_or_default<V: Clone + Default + Send + Sync + 'static>(
        self: &Arc<Self>,
        key: &str,
    ) -> V {
        self.runtime.get_static::<V>(key).unwrap_or_default()
    }

    /// Insert one already-built route definition accepted by this app role.
    pub fn insert<P, H>(self: &Arc<Self>, def: AccessPointDef<P, H>) -> Result<(), BindError>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        H: FinalHandlerDef<P>,
        T: Accepts<H>,
    {
        self.registry.register(&def)
    }

    /// Calls the generated constructor exactly once and registers its result.
    pub fn bind<P, H>(
        self: &Arc<Self>,
        constructor: fn() -> AccessPointDef<P, H>,
    ) -> Result<(), BindError>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        H: FinalHandlerDef<P>,
        T: Accepts<H>,
    {
        self.insert(constructor())
    }

    /// Applies retained routes to this App's existing protocol entries.
    /// It never creates a protocol entry.
    pub fn apply(self: &Arc<Self>, blueprint: &Blueprint<TS, T>) -> Result<(), BlueprintError> {
        // Full preflight prevents missing-protocol partial mutation.
        for group in blueprint.groups() {
            if !group.has_entry(&self.registry) {
                return Err(BlueprintError::ProtocolNotFound(group.protocol_name()));
            }
        }
        for group in blueprint.groups() {
            group.register_into(&self.registry)?;
        }
        Ok(())
    }

    /// Insert a homogeneous batch of already-built definitions, stopping at
    /// the first error.
    pub fn extend<P, H, I>(self: &Arc<Self>, defs: I) -> Result<(), BindError>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        H: FinalHandlerDef<P>,
        T: Accepts<H>,
        I: IntoIterator<Item = AccessPointDef<P, H>>,
    {
        for (index, def) in defs.into_iter().enumerate() {
            self.registry
                .register(&def)
                .map_err(|error| error.with_batch_index(index))?;
        }
        Ok(())
    }

    // `App::lit_url` and `App::url` (raw `ExecutableBinding` registration)
    // are intentionally removed in the Stage-5 AP registration cleanup. The
    // canonical path is `App::insert` / `App::extend` (or the
    // `App::bind(constructor)` wrapper) with an `AccessPointDef`, which
    // funnels through `ProtocolRegistryKind::register` ->
    // `ProtocolEntry::register`. This knowingly breaks the old trans `url`
    // pipeline until the Stage-10 cutover; do not re-add a raw registry
    // wrapper to keep them alive.

    /// Left-biased merge of two exclusively owned apps with the same target.
    pub fn try_combine(
        self: Arc<Self>,
        other: Arc<Self>,
    ) -> Result<Arc<Self>, AppInUse<Arc<Self>>> {
        if Arc::ptr_eq(&self, &other) {
            return Ok(self);
        }

        let this = match Arc::try_unwrap(self) {
            Ok(inner) => inner,
            Err(shared) => {
                debug_warn!("App::try_combine refused: self is shared");
                return Err(AppInUse { app: shared, other });
            }
        };
        let other = match Arc::try_unwrap(other) {
            Ok(inner) => inner,
            Err(shared) => {
                debug_warn!("App::try_combine refused: other is shared");
                return Err(AppInUse {
                    app: Arc::new(this),
                    other: shared,
                });
            }
        };

        let runtime = match Arc::try_unwrap(this.runtime) {
            Ok(mut runtime) => {
                if let Ok(other_runtime) = Arc::try_unwrap(other.runtime) {
                    runtime.combine(other_runtime);
                }
                Arc::new(runtime)
            }
            Err(shared) => shared,
        };

        Ok(Arc::new(Self {
            registry: this.registry.combine(other.registry),
            inbound_state: this.inbound_state,
            outbound_state: this.outbound_state,
            runtime,
            config: this.config.combine(other.config),
            _rt: PhantomData,
            _target: PhantomData,
        }))
    }

    /// Combines two apps, returning `self` unchanged when either app is shared.
    pub fn combine(self: Arc<Self>, other: Arc<Self>) -> Arc<Self> {
        self.try_combine(other).unwrap_or_else(|error| error.app)
    }

    /// Retries `try_combine` until both apps are exclusively owned.
    pub async fn combine_wait(self: Arc<Self>, other: Arc<Self>) -> Arc<Self> {
        let (mut app, mut other) = (self, other);
        loop {
            match app.try_combine(other) {
                Ok(combined) => return combined,
                Err(error) => {
                    debug_log!("App::combine_wait: app Arc still shared; retrying");
                    (app, other) = (error.app, error.other);
                    Rt::sleep(Duration::from_millis(10)).await;
                }
            }
        }
    }
}
