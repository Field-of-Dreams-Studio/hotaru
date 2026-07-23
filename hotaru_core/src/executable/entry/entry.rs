use crate::prelude::Arc;
#[cfg(not(feature = "std"))]
use crate::prelude::*;
use core::{any::Any, time::Duration};

use crate::{
    alias::PRwLock,
    connection::{
        BufferedReadHalf, BufferedWriteHalf, HotaruRead, HotaruWrite, MaybeSendBoxFuture,
    },
};

use crate::protocol::{Channel, ProtocolFlow};
use crate::{
    app::common::RuntimeConfig,
    connection::{ConnStream, TransportSpec},
    executable::{
        ExecutableBinding,
        access::{access_point::AccessPoint, table::AccessPointTable},
        def::{AccessPointDef, BindError, FinalHandlerDef},
        entry::ProtocolEntryTrait,
        middleware::AsyncMiddlewareChain,
    },
    protocol::Protocol,
    url::{PathPattern, UrlError, UrlRegistration, UrlRoot, node::StepName},
};
use akari::extensions::{Locals, Params, ParamsClone};

/// Concrete handler for a specific protocol.
pub struct ProtocolEntry<P, TS>
where
    P: Protocol<Wire = TS::Wire, TS = TS> + Clone,
    TS: TransportSpec,
{
    pub protocol: P,
    pub root_handler: Arc<UrlRoot<P::Context, TS>>,
    pub middlewares: AsyncMiddlewareChain<P::Context>,
    pub access_points: AccessPointTable<P::Context, TS>,
}

impl<P, TS> ProtocolEntry<P, TS>
where
    P: Protocol<Wire = TS::Wire, TS = TS> + Clone,
    TS: TransportSpec,
{
    pub fn new(
        protocol: P,
        root_handler: Arc<UrlRoot<P::Context, TS>>,
        middlewares: AsyncMiddlewareChain<P::Context>,
    ) -> Self {
        Self {
            protocol,
            root_handler,
            middlewares,
            access_points: AccessPointTable::new(),
        }
    }

    /// Register a binding at the given pre-parsed path under `name`, and
    /// refresh any existing access-point entries pointing at that path.
    ///
    /// This is the single canonical Layer-1 funnel. Pattern parsing (full
    /// Hotaru grammar) and literal splitting are both wrapper-level concerns
    /// — by the time we get here the caller has already chosen the parsing
    /// strategy and produced the pre-parsed `Vec<PathPattern>` plus
    /// `step_names` metadata.
    ///
    /// `step_names` carries named-capture metadata from pattern parsing
    /// (e.g. the `id` in `/users/<id>`). Pass `StepName::default()` for
    /// purely literal paths that have no captures.
    ///
    /// Low-level primitive: the caller already produced the pre-parsed path
    /// and a finished `ExecutableBinding`. `register` (the `AccessPointDef`
    /// path) is the normal entry point and funnels through here.
    pub(crate) fn register_internal<N: Into<String>>(
        &self,
        name: N,
        path: Vec<PathPattern>,
        step_names: StepName,
        binding: ExecutableBinding<P::Context>,
        config: ParamsClone,
    ) -> Result<UrlRegistration<P::Context, TS>, UrlError> {
        let reg = self
            .root_handler
            .register(path.clone(), binding, config, step_names)?;
        if let UrlRegistration::Node(arc) = &reg {
            self.access_points.refresh_path(&path, arc);
        }
        self.access_points.insert(
            name,
            AccessPoint {
                path,
                target: reg.clone(),
            },
        );
        Ok(reg)
    }

    /// Compile one `AccessPointDef` and register the resulting binding.
    ///
    /// URL parsing lives on the definition; middleware-slot resolution runs
    /// against this entry's own protocol-root snapshot. This is the single
    /// canonical `AccessPointDef` registration path; `register_internal`
    /// stays the low-level parsed-path primitive it funnels through.
    pub(crate) fn register<H>(&self, def: &AccessPointDef<P, H>) -> Result<(), BindError>
    where
        H: FinalHandlerDef<P>,
    {
        let (path, step_names) = def.parse_url_pattern()?;
        let middlewares = def
            .middlewares()
            .resolve(&self.middlewares, def.handler().body_middleware());
        let binding = ExecutableBinding::new()
            .with_handler(def.handler().final_handler())
            .with_middlewares(middlewares);
        self.register_internal(def.name(), path, step_names, binding, def.config().clone())
            .map(|_| ())
            .map_err(|error| BindError::new(def.name(), def.url(), error))
    }

    /// Wrap an acquired wire in this protocol's channel handle.
    /// The caller (e.g. `Client::open_channel`) owns connection sourcing.
    pub fn create_channel(&self, wire: TS::Wire) -> P::Channel {
        let (read, write, meta) = wire.split();
        let reader = read.into_buf();
        let writer = write.into_buf_write();
        self.protocol.clone().open_channel(reader, writer, meta)
    }
}

impl<P, TS> ProtocolEntryTrait<TS> for ProtocolEntry<P, TS>
where
    P: Protocol<Wire = TS::Wire, TS = TS> + Clone + 'static,
    TS: TransportSpec,
{
    fn test(&self, buf: &[u8]) -> bool {
        P::detect(buf)
    }

    fn default_connection_timeout(&self) -> Option<Duration> {
        self.protocol.default_connection_timeout()
    }

    fn serve(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufferedReadHalf<TS>,
        writer: BufferedWriteHalf<TS>,
        meta: <TS::Wire as ConnStream>::Meta,
    ) -> MaybeSendBoxFuture<'static, ()> {
        let protocol = self.protocol.clone();
        let root = self.root_handler.clone();

        Box::pin(async move {
            let channel = protocol.open_channel(reader, writer, meta);
            while channel.is_open() {
                match P::handle(&channel, runtime.clone(), root.clone()).await {
                    Ok(ProtocolFlow::Continue) => continue,
                    Ok(ProtocolFlow::Close) => {
                        channel.close();
                    }
                    Err(_e) => {
                        channel.close();
                        return;
                    }
                }
            }
        })
    }

    fn serve_upgrade(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufferedReadHalf<TS>,
        writer: BufferedWriteHalf<TS>,
        meta: <TS::Wire as ConnStream>::Meta,
        _params: PRwLock<Params>,
        _locals: PRwLock<Locals>,
    ) -> MaybeSendBoxFuture<'static, ()> {
        self.serve(runtime, reader, writer, meta)
    }

    fn request(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufferedReadHalf<TS>,
        writer: BufferedWriteHalf<TS>,
        meta: <TS::Wire as ConnStream>::Meta,
    ) -> MaybeSendBoxFuture<'static, ()> {
        let protocol = self.protocol.clone();
        let root = self.root_handler.clone();

        Box::pin(async move {
            let channel = protocol.open_channel(reader, writer, meta);
            while channel.is_open() {
                match P::handle(&channel, runtime.clone(), root.clone()).await {
                    Ok(ProtocolFlow::Continue) => continue,
                    Ok(ProtocolFlow::Close) => {
                        channel.close();
                    }
                    Err(_e) => {
                        channel.close();
                        return;
                    }
                }
            }
        })
    }

    fn request_upgrade(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufferedReadHalf<TS>,
        writer: BufferedWriteHalf<TS>,
        meta: <TS::Wire as ConnStream>::Meta,
        _params: PRwLock<Params>,
        _locals: PRwLock<Locals>,
    ) -> MaybeSendBoxFuture<'static, ()> {
        self.request(runtime, reader, writer, meta)
    }

    fn combine_from(&self, other: &dyn ProtocolEntryTrait<TS>) -> bool {
        let Some(other) = other.as_any().downcast_ref::<Self>() else {
            return false;
        };
        self.root_handler.combine(&other.root_handler);
        for name in other.access_points.names() {
            if !self.access_points.contains(&name) {
                if let Some(ap) = other.access_points.get(&name) {
                    self.access_points.insert(name, ap);
                }
            }
        }
        // middlewares: left-biased, self's chain kept untouched
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
