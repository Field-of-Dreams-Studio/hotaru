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
use crate::prelude::Arc;
use core::any::TypeId;
use core::time::Duration;

use crate::{
    app::common::RuntimeConfig,
    connection::{ConnStream, HotaruRead, HotaruWrite, TransportSpec},
    executable::{
        ExecutableBinding,
        def::{AccessPointDef, BindError, FinalHandlerDef, MWChain},
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

    /// Compile an access-point definition and register the resulting binding.
    ///
    /// URL parsing and middleware-slot resolution stay with their owning
    /// definition types; this method supplies the registry-owned middleware
    /// snapshot and registration operation.
    pub(crate) fn compile_and_register<P, T>(
        &self,
        def: AccessPointDef<P, T>,
    ) -> Result<(), BindError>
    where
        P: Protocol<Wire = TS::Wire, TS = TS> + 'static,
        T: FinalHandlerDef<P>,
    {
        let (path, step_names) = def.parse_url_pattern()?;
        let inherited = self.get_protocol_middlewares::<P>();
        let (address, middleware_slots, handler, config) = def.into_parts();
        let (url, name, _) = address.into_parts();

        let middlewares = MWChain::into_chain(
            middleware_slots,
            &inherited,
            handler.body_middleware(),
        );
        let binding = ExecutableBinding::new()
            .with_handler(handler.final_handler())
            .with_middlewares(middlewares);

        self.register::<P, _>(name.as_str(), path, step_names, binding, config)
            .map(|_| ())
            .map_err(|error| BindError::new(name, url, error))
    }

    /// Merges two registry wrappers, re-optimizing Single/Multi afterwards.
    pub fn combine(self, other: Self) -> Self {
        let mut merged = self.into();
        merged.combine(other.into());
        Self::from(merged)
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

#[cfg(test)]
mod tests {
    use crate::prelude::Arc;
    use core::convert::Infallible;
    use core::future::Future;

    use akari::extensions::ParamsClone;

    use crate::{
        connection::{
            MaybeSend,
            test_support::{TestMeta, TestOutbound, TestTransport, TestWire},
        },
        executable::middleware::AsyncFinalHandler,
        protocol::{Channel, DefaultProtocolError, ProtocolFlow, ProtocolRole},
        url::{PathPattern, UrlRoot},
    };

    use super::*;

    #[derive(Debug)]
    struct TestError;

    impl core::fmt::Display for TestError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("test error")
        }
    }

    impl core::error::Error for TestError {}
    impl DefaultProtocolError for TestError {}

    impl From<Infallible> for TestError {
        fn from(x: Infallible) -> Self {
            match x {}
        }
    }

    #[derive(Clone)]
    enum NoChannel {}

    impl Channel for NoChannel {
        fn is_open(&self) -> bool {
            match *self {}
        }
        fn close(&self) {}
    }

    #[derive(Default)]
    struct TestCtx;

    impl crate::protocol::RequestContext for TestCtx {
        type Request = ();
        type Response = ();
        type Error = TestError;
        type Channel = NoChannel;

        fn handle_error(&mut self) {}
        fn role(&self) -> ProtocolRole {
            ProtocolRole::Server
        }
        fn inject_request(&mut self, _: ()) {}
        fn into_response(self) {}
    }

    /// Type-only test protocol; `N` distinguishes concrete protocol types.
    #[derive(Clone)]
    struct TestProto<const N: u8>;

    impl<const N: u8> Protocol for TestProto<N> {
        type Wire = TestWire;
        type TS = TestTransport;
        type Channel = NoChannel;
        type Stream = ();
        type Message = ();
        type Context = TestCtx;

        fn name(&self) -> &'static str {
            "test-proto"
        }
        fn role(&self) -> ProtocolRole {
            ProtocolRole::Server
        }
        fn detect(_: &[u8]) -> bool {
            true
        }

        fn open_channel(self, reader: TestWire, _writer: TestWire, _meta: TestMeta) -> NoChannel {
            match reader {}
        }

        fn handle(
            _channel: &NoChannel,
            _runtime: Arc<RuntimeConfig>,
            _root: Arc<UrlRoot<TestCtx, TestTransport>>,
        ) -> impl Future<Output = Result<ProtocolFlow, TestError>> + MaybeSend {
            async { Ok(ProtocolFlow::Close) }
        }

        fn acquire_channel(
            &self,
            _runtime: &Arc<RuntimeConfig>,
            _outbound: Arc<TestOutbound>,
        ) -> impl Future<Output = Result<NoChannel, TestError>> + MaybeSend {
            async { Err(TestError) }
        }

        fn send(ctx: TestCtx) -> impl Future<Output = Result<TestCtx, TestError>> + MaybeSend {
            async move { Ok(ctx) }
        }

        fn install_channel(_ctx: &mut TestCtx, channel: NoChannel) {
            match channel {}
        }
    }

    fn binding_with_handler() -> ExecutableBinding<TestCtx> {
        let handler: Arc<dyn AsyncFinalHandler<TestCtx>> =
            Arc::new(|ctx: TestCtx| async move { Ok(ctx) });
        ExecutableBinding::new().with_handler(handler)
    }

    fn single<const N: u8>(route: &str, ap_name: &str) -> ProtocolRegistryKind<TestTransport> {
        let kind = ProtocolRegistryKind::single(
            TestProto::<N>,
            Arc::new(UrlRoot::new()),
            vec![],
        );
        kind.register::<TestProto<N>, _>(
            ap_name,
            vec![PathPattern::literal_path(route)],
            StepName::default(),
            binding_with_handler(),
            ParamsClone::default(),
        )
        .unwrap();
        kind
    }

    /// End-to-end combine through every layer: kind → registry →
    /// `combine_from` → URL tree + access-point table.
    #[tokio::test]
    async fn kind_combine_resolves_same_protocol_and_appends_new() {
        let left = single::<0>("left", "shared");
        let right = single::<0>("right", "shared");
        right
            .register::<TestProto<0>, _>(
                "extra",
                vec![PathPattern::literal_path("extra")],
                StepName::default(),
                binding_with_handler(),
                ParamsClone::default(),
            )
            .unwrap();

        // Same protocol: stays Single, routes union, APs merge left-biased.
        let merged = left.combine(right);
        assert!(matches!(merged, ProtocolRegistryKind::Single(_)));

        let root = merged.url::<TestProto<0>>().unwrap();
        assert!(root.walk_str("left").await.is_some());
        assert!(root.walk_str("right").await.is_some());

        let entry = merged.entry::<TestProto<0>>().unwrap();
        let shared = entry.access_points.get("shared").unwrap();
        assert_eq!(shared.path, vec![PathPattern::literal_path("left")]);
        assert!(entry.access_points.contains("extra"));
        assert_eq!(entry.access_points.len(), 2);

        // Different protocol: appended, re-optimized to Multi.
        let merged = merged.combine(single::<1>("other", "other"));
        assert!(matches!(merged, ProtocolRegistryKind::Multi(_)));
        assert!(merged.url::<TestProto<0>>().is_some());
        assert!(merged.url::<TestProto<1>>().is_some());
    }
}
