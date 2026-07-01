//! Shared protocol-registry wrapper used by both `Server` and `Client`.
//!
//! This file replaces the two near-duplicate `app/server/registry.rs` and
//! `app/client/registry.rs` files. The wrapper is a small "is this one
//! protocol or many" optimization layered on top of
//! [`ProtocolEntryRegistry`]. Both `Server` and `Client` use it identically,
//! diverging only in their dispatch verb:
//!
//! - `Server` calls [`ProtocolRegistryKind::serve`] from its inbound loop.
//! - `Client` calls [`ProtocolRegistryKind::request`] from its outbound path.
//!
//! `default_connection_timeout` is exposed for server-side `TimeoutSetting::Inherit`
//! resolution; client code does not call it.
//!
//! TODO: Most helper methods here just delegate to the inner
//! [`ProtocolEntryRegistry`]. Once the Stage-4 registration funnel lands, the
//! helpers (`url`, `lit_url`, `sub_url`, `get_protocol_middlewares`, …) should
//! migrate down into `executable::registry::ProtocolEntryRegistry` and this
//! wrapper should shrink to just the enum + `from`/`into` + the two
//! dispatchers + `default_connection_timeout`.

#[cfg(not(feature = "std"))]
use crate::prelude::*;
use alloc::sync::Arc;
use core::any::TypeId;
use core::time::Duration;

use crate::{
    app::common::RuntimeConfig,
    connection::{ConnStream, HotaruRead, HotaruWrite, TransportSpec},
    executable::{
        ExecutableBinding,
        entry::{ProtocolEntry, ProtocolEntryTrait},
        middleware::{AsyncMiddleware, AsyncMiddlewareChain},
        registry::ProtocolEntryRegistry,
    },
    extensions::ParamsClone,
    protocol::Protocol,
    url::{PathPattern, UrlError, UrlRegistration, UrlRoot, node::StepName},
};

/// Optimization for single-protocol apps, which are common in practice.
///
/// `Single` skips the protocol-detection loop when only one protocol is
/// registered; `Multi` falls back to the full registry.
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

    // ------------------------------------------------------------------
    // Dispatch — Server-side
    // ------------------------------------------------------------------

    /// Server-side dispatch: detect the protocol on an inbound wire and
    /// hand it to the matching entry's `serve` method. Called from
    /// `Server::handle_wire`.
    pub async fn serve(&self, runtime: Arc<RuntimeConfig>, conn: TS::Wire) {
        match self {
            Self::Single(handler) => {
                let (reader, writer, meta) = conn.split();
                let reader = reader.into_buf();
                let writer = writer.into_buf_write();
                handler.serve(runtime, reader, writer, meta).await;
            }
            Self::Multi(registry) => {
                registry.serve(runtime, conn).await;
            }
        }
    }

    // ------------------------------------------------------------------
    // Dispatch — Client-side
    // ------------------------------------------------------------------

    /// Client-side dispatch: detect the protocol on an outbound wire and
    /// hand it to the matching entry's `request` method. Called from
    /// `Client::run_wire`.
    pub async fn request(&self, runtime: Arc<RuntimeConfig>, conn: TS::Wire) {
        match self {
            Self::Single(handler) => {
                let (reader, writer, meta) = conn.split();
                let reader = reader.into_buf();
                let writer = writer.into_buf_write();
                handler.request(runtime, reader, writer, meta).await;
            }
            Self::Multi(registry) => {
                registry.request(runtime, conn).await;
            }
        }
    }

    // ------------------------------------------------------------------
    // URL registration helpers (currently duplicated with inner registry —
    // see file-level TODO)
    // ------------------------------------------------------------------

    /// Find the root URL node for a protocol, if registered. Used by URL registration
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

    /// Find the protocol entry for a protocol, if registered. Else, returns `None`.
    pub fn entry<P: Protocol<Wire = TS::Wire, TS = TS> + 'static>(
        &self,
    ) -> Option<&ProtocolEntry<P, TS>> {
        match self {
            Self::Single(handler) => handler.as_any().downcast_ref::<ProtocolEntry<P, TS>>(),
            Self::Multi(registry) => {
                for handler in &registry.handlers {
                    if let Some(ph) = handler.as_any().downcast_ref::<ProtocolEntry<P, TS>>() {
                        return Some(ph);
                    }
                }
                None
            }
        }
    }

    /// The canonical registration funnel. Caller passes a name, pre-parsed
    /// path segments, and step-name metadata. Routes to the matching
    /// concrete `ProtocolEntry<P>`'s `register` method, which updates both
    /// the URL tree and the entry's `AccessPointTable`.
    ///
    /// Both pattern parsing (`url::parser::parse`) and literal splitting
    /// are wrapper-level concerns done by `Server::url` / `Server::lit_url`
    /// (and the `Client` mirrors) before reaching here.
    pub fn register<P, N>(
        &self,
        name: N,
        path: Vec<PathPattern>,
        step_names: StepName,
        executable: ExecutableBinding<P::Context>,
        config: ParamsClone,
    ) -> Result<UrlRegistration<P::Context, TS>, UrlError>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        N: Into<String>,
    {
        let entry = self.entry::<P>().ok_or(UrlError::ProtocolNotFound)?;
        entry.register(name, path, step_names, executable, config)
    }

    #[av::ver(
        deprecated,
        since = "0.8.0",
        note = "Use `register` after parsing the path. This method bypasses the protocol entry's AccessPointTable and will silently orphan named registrations from the freshness logic."
    )]
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

    #[av::ver(
        deprecated,
        since = "0.8.0",
        note = "Use `register` after parsing the path. This method bypasses the protocol entry's AccessPointTable and will silently orphan named registrations from the freshness logic."
    )]
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

    // ------------------------------------------------------------------
    // Server-only: timeout resolution
    // ------------------------------------------------------------------

    /// Returns the default connection-timeout to use when [`TimeoutSetting::Inherit`](crate::app::common::TimeoutSetting::Inherit)
    /// is configured.
    ///
    /// For `Single`, delegates directly to the protocol.
    ///
    /// For `Multi`, returns the **longest** default across all registered
    /// protocols so no protocol's connections are prematurely cut off. `None`
    /// (no timeout) beats any finite duration; among finite durations the
    /// maximum wins.
    ///
    /// Client code does not call this — the client side uses
    /// `OperationalConfig.connect_timeout` and `request_timeout` instead of
    /// `max_connection_time`. The method lives on the shared wrapper for
    /// convenience; it's harmless on the client.
    ///
    /// TODO: This is an interim heuristic. The correct fix is to resolve
    /// `Inherit` *after* protocol detection so each connection uses the
    /// matched protocol's own default. That requires moving timeout
    /// application inside the serve path.
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

    // ------------------------------------------------------------------
    // Introspection helpers
    // ------------------------------------------------------------------

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
