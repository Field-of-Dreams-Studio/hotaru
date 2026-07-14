//! Unified application storage shared by inbound, outbound, and dual-role apps.

use core::marker::PhantomData;
use core::time::Duration;

use akari::extensions::ParamsClone;

use crate::app::common::{AppInUse, OperationalConfig, RunMode, RuntimeConfig, TimeoutSetting};
use crate::app::registry::ProtocolRegistryKind;
use crate::app::runtime::RuntimeSpec;
use crate::connection::TransportSpec;
use crate::executable::ExecutableBinding;
use crate::prelude::{Arc, String, Vec};
use crate::protocol::Protocol;
use crate::url::{PathPattern, UrlError, node::StepName};
use crate::{debug_log, debug_warn};

use super::target::AppTarget;

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

    /// Returns the configured maximum request processing time in seconds.
    pub fn get_max_frame_process_time(self: &Arc<Self>) -> usize {
        self.config.max_frame_process_time()
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

    /// Registers a literal URL without applying the pattern grammar.
    pub fn lit_url<P, U, N>(
        self: &Arc<Self>,
        url: U,
        name: N,
        mut executable: ExecutableBinding<P::Context>,
        config: ParamsClone,
    ) -> Result<(), UrlError>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        U: AsRef<str>,
        N: Into<String>,
    {
        if executable.has_no_middlewares() {
            executable.set_middlewares(self.registry.get_protocol_middlewares::<P>());
        }

        let path: Vec<PathPattern> = P::lit_parser(url.as_ref())
            .into_iter()
            .map(PathPattern::literal_path)
            .collect();

        self.registry
            .register::<P, _>(name, path, StepName::default(), executable, config)?;
        Ok(())
    }

    /// Registers a URL using the protocol's pattern grammar.
    pub fn url<P, U, N>(
        self: &Arc<Self>,
        url: U,
        name: N,
        mut executable: ExecutableBinding<P::Context>,
        config: ParamsClone,
    ) -> Result<(), UrlError>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        U: AsRef<str>,
        N: Into<String>,
    {
        if executable.has_no_middlewares() {
            executable.set_middlewares(self.registry.get_protocol_middlewares::<P>());
        }

        let tokens = P::tokenize_url(url.as_ref())?;
        let (path, step_names) = crate::url::tokens_to_patterns(&tokens)?;
        self.registry
            .register::<P, _>(name, path, step_names.into(), executable, config)?;
        Ok(())
    }

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
