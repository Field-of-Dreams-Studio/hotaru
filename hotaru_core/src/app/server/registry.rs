use std::time::Duration;

use crate::{
    app::common::RuntimeConfig,
    connection::{ConnStream,TransportSpec},
    executable::{
        ExecutableBinding,
        entry::{ProtocolEntry, ProtocolEntryTrait},
        middleware::{AsyncMiddleware, AsyncMiddlewareChain},
        registry::ProtocolEntryRegistry,
    },
    extensions::ParamsClone,
    protocol::Protocol,
    url::{UrlError, UrlRegistration, UrlRoot},
};
use std::{any::TypeId, sync::Arc};
use tokio::io::BufReader;

/// Optimization for single-protocol servers, which are common in practice.
pub enum ProtocolRegistryKind<TS: TransportSpec> {
    Single(Arc<dyn ProtocolEntryTrait<TS>>),
    Multi(ProtocolEntryRegistry<TS>),
}

impl<TS: TransportSpec> ProtocolRegistryKind<TS> {
    /// Builds the optimized wrapper from the neutral entry registry.
    pub fn from(registry: ProtocolEntryRegistry<TS>) -> Self {
        let mut handlers = registry.handlers;
        match handlers.len() {
            1 => Self::Single(handlers.pop().unwrap()),
            _ => Self::Multi(ProtocolEntryRegistry { handlers }),
        }
    }

    /// Converts the optimized wrapper back into the neutral entry registry.
    pub fn into(self) -> ProtocolEntryRegistry<TS> {
        match self {
            Self::Single(handler) => ProtocolEntryRegistry {
                handlers: vec![handler],
            },
            Self::Multi(registry) => registry,
        }
    }

    // TODO: Most helper methods below are duplicated in app/client/registry.rs.
    // Once client/server wrappers settle, keep only serve-side dispatch here and
    // move shared helper methods down into executable::registry::ProtocolEntryRegistry.
    pub fn single<P: Protocol<Wire = TS::Wire, TS = TS> + Clone + 'static>(
        protocol: P,
        root_handler: Arc<UrlRoot<P::Context, TS>>,
        middlewares: AsyncMiddlewareChain<P::Context>,
    ) -> Self {
        Self::Single(Arc::new(ProtocolEntry::new(
            protocol,
            root_handler,
            middlewares,
        )))
    }

    pub fn multi(registry: ProtocolEntryRegistry<TS>) -> Self {
        Self::Multi(registry)
    }

    pub async fn run(&self, runtime: Arc<RuntimeConfig>, conn: TS::Wire) {
        match self {
            Self::Single(handler) => {
                let (reader, writer, meta) = conn.split();
                let reader = BufReader::new(reader);
                handler.serve(runtime, reader, writer, meta).await;
            }
            Self::Multi(registry) => {
                registry.serve(runtime, conn).await;
            }
        }
    }

    pub fn url<P: Protocol<Wire = TS::Wire, TS = TS> + 'static>(
        &self,
    ) -> Option<Arc<UrlRoot<P::Context, TS>>> {
        match self {
            Self::Single(handler) => handler
                .as_any()
                .downcast_ref::<ProtocolEntry<P, TS>>()
                .map(|ph| ph.root_handler.clone()),
            Self::Multi(registry) => {
                for handler in &registry.handlers {
                    if let Some(ph) = handler.as_any().downcast_ref::<ProtocolEntry<P, TS>>() {
                        return Some(ph.root_handler.clone());
                    }
                }
                None
            }
        }
    }

    pub fn lit_url<P: Protocol<Wire = TS::Wire, TS = TS> + 'static, T: Into<String>>(
        &self,
        url: T,
        executable: ExecutableBinding<P::Context>,
        config: ParamsClone,
    ) -> Result<UrlRegistration<P::Context, TS>, UrlError> {
        let url = url.into();
        match self
            .url::<P>()
            .map(|root| root.literal_url(&url, executable, config))
        {
            Some(result) => result,
            None => Err(UrlError::ProtocolNotFound),
        }
    }

    pub fn sub_url<P: Protocol<Wire = TS::Wire, TS = TS> + 'static, T: Into<String>>(
        &self,
        pattern: T,
        executable: ExecutableBinding<P::Context>,
        config: ParamsClone,
    ) -> Result<UrlRegistration<P::Context, TS>, UrlError> {
        let pattern = pattern.into();
        match self
            .url::<P>()
            .map(|root| root.sub_url(&pattern, executable, config))
        {
            Some(result) => result,
            None => Err(UrlError::ProtocolNotFound),
        }
    }

    /// Returns the default connection-timeout to use when [`TimeoutSetting::Inherit`]
    /// is configured.
    ///
    /// For `Single`, delegates directly to the protocol.
    ///
    /// For `Multi`, returns the **longest** default across all registered protocols
    /// so no protocol's connections are prematurely cut off. `None` (no timeout)
    /// beats any finite duration; among finite durations the maximum wins.
    ///
    /// TODO: This is an interim heuristic. The correct fix is to resolve `Inherit`
    /// *after* protocol detection so each connection uses the matched protocol's
    /// own default. That requires moving timeout application inside the serve path.
    pub fn default_connection_timeout(&self) -> Option<Duration> {
        match self {
            Self::Single(handler) => handler.default_connection_timeout(),
            Self::Multi(registry) => {
                let mut longest: Option<Duration> = Some(Duration::ZERO);
                for h in &registry.handlers {
                    match (longest, h.default_connection_timeout()) {
                        // None (infinite) beats everything.
                        (_, None) => return None,
                        // Accumulate the maximum finite duration.
                        (Some(acc), Some(d)) => longest = Some(acc.max(d)),
                        // Already infinite — unreachable given the None arm above,
                        // but silence the exhaustiveness warning.
                        (None, _) => {}
                    }
                }
                // Empty registry: fall back to a safe 30-second default.
                if longest == Some(Duration::ZERO) && registry.handlers.is_empty() {
                    Some(Duration::from_secs(30))
                } else {
                    longest
                }
            }
        }
    }

    pub fn first_protocol_type_id(&self) -> Option<TypeId> {
        match self {
            Self::Single(handler) => Some(handler.as_any().type_id()),
            Self::Multi(registry) => registry.handlers.first().map(|h| h.as_any().type_id()),
        }
    }

    pub fn get_protocol_middlewares<P: Protocol<Wire = TS::Wire, TS = TS> + 'static>(
        &self,
    ) -> Vec<Arc<dyn AsyncMiddleware<P::Context>>> {
        match self {
            Self::Single(handler) => {
                if let Some(protocol_entry) =
                    handler.as_any().downcast_ref::<ProtocolEntry<P, TS>>()
                {
                    protocol_entry.middlewares.clone()
                } else {
                    vec![]
                }
            }
            Self::Multi(registry) => {
                for handler in &registry.handlers {
                    if let Some(protocol_entry) =
                        handler.as_any().downcast_ref::<ProtocolEntry<P, TS>>()
                    {
                        return protocol_entry.middlewares.clone();
                    }
                }
                vec![]
            }
        }
    }
}
