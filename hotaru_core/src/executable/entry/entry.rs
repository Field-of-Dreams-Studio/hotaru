use core::{any::Any, future::Future, pin::Pin, time::Duration};
use std::sync::{Arc, RwLock};

use akari::extensions::{Locals, Params, ParamsClone};
use tokio::io::BufReader;

use crate::{
    app::common::RuntimeConfig, connection::{ConnStream, TransportSpec}, executable::{ExecutableBinding, access::{access_point::AccessPoint, table::AccessPointTable}, entry::ProtocolEntryTrait, middleware::AsyncMiddlewareChain}, protocol::Protocol, url::{PathPattern, UrlError, UrlRegistration, UrlRoot, node::StepName}
};
use crate::protocol::{Channel, ProtocolFlow};

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
    pub fn register<N: Into<String>>(
        &self,
        name: N,
        path: Vec<PathPattern>,
        step_names: StepName,
        binding: ExecutableBinding<P::Context>,
        config: ParamsClone,
    ) -> Result<UrlRegistration<P::Context, TS>, UrlError> {
        let reg = self.root_handler.register(path.clone(), binding, config, step_names)?;
        if let UrlRegistration::Node(arc) = &reg {
            self.access_points.refresh_path(&path, arc);
        }
        self.access_points.insert(
            name,
            AccessPoint { path, target: reg.clone() },
        );
        Ok(reg)
    }

    /// Wrap an acquired wire in this protocol's channel handle.
    /// The caller (e.g. `Client::open_channel`) owns connection sourcing.
    pub fn create_channel(&self, wire: TS::Wire) -> P::Channel {
        let (read, write, meta) = wire.split();
        let reader = BufReader::new(read);
        self.protocol.clone().open_channel(reader, write, meta)
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
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
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
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
        _params: RwLock<Params>,
        _locals: RwLock<Locals>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        self.serve(runtime, reader, writer, meta)
    }

    fn request(
        &self,
        runtime: Arc<RuntimeConfig>,
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
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
        reader: BufReader<<TS::Wire as ConnStream>::ReadHalf>,
        writer: <TS::Wire as ConnStream>::WriteHalf,
        meta: <TS::Wire as ConnStream>::Meta,
        _params: RwLock<Params>,
        _locals: RwLock<Locals>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        self.request(runtime, reader, writer, meta)
    } 

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
