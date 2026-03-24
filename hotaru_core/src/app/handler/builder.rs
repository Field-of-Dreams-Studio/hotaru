use super::{ProtocolHandler, ProtocolHandlerTrait, ProtocolRegistry, ProtocolRegistryKind};
use crate::{
    connection::{Protocol, TransportSpec},
    executable::middleware::AsyncMiddleware,
    url::Url,
};
use std::sync::Arc;

/// Builder for protocol handlers.
pub struct ProtocolHandlerBuilder<P, TS = crate::connection::tcp::TcpTransport>
where
    P: Protocol<Wire = TS::Wire> + Clone + 'static,
    TS: TransportSpec,
{
    protocol: Option<P>,
    url: Arc<Url<P::Context, TS>>,
    middlewares: Vec<Arc<dyn AsyncMiddleware<P::Context>>>,
}

impl<P, TS> ProtocolHandlerBuilder<P, TS>
where
    P: Protocol<Wire = TS::Wire> + Clone + 'static,
    TS: TransportSpec,
{
    pub fn new(protocol: P) -> Self {
        Self {
            protocol: Some(protocol),
            url: Arc::new(Url::<P::Context, TS>::default()),
            middlewares: Vec::new(),
        }
    }

    pub fn set_url(mut self, url: Arc<Url<P::Context, TS>>) -> Self {
        self.url = url;
        self
    }

    pub fn append_middleware<M>(mut self) -> Self
    where
        M: AsyncMiddleware<P::Context> + 'static,
    {
        self.middlewares.push(Arc::new(M::return_self()));
        self
    }

    pub fn build(self) -> Arc<dyn ProtocolHandlerTrait<TS>> {
        Arc::new(ProtocolHandler::new(
            self.protocol.expect("Protocol must be set"),
            self.url,
            self.middlewares,
        ))
    }
}

pub struct ProtocolRegistryBuilder<TS: TransportSpec> {
    handlers: Vec<Arc<dyn ProtocolHandlerTrait<TS>>>,
}

impl<TS: TransportSpec> ProtocolRegistryBuilder<TS> {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    pub fn protocol<P: Protocol<Wire = TS::Wire> + Clone + 'static>(
        mut self,
        builder: ProtocolHandlerBuilder<P, TS>,
    ) -> Self {
        self.handlers.push(builder.build());
        self
    }

    pub fn build(self) -> ProtocolRegistryKind<TS> {
        match self.handlers.len() {
            1 => ProtocolRegistryKind::Single(self.handlers.into_iter().next().unwrap()),
            _ => ProtocolRegistryKind::Multi(ProtocolRegistry {
                handlers: self.handlers,
            }),
        }
    }
}
