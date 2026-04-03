use std::sync::Arc;

use crate::{
    connection::{Protocol, TransportSpec},
    executable::entry::{ProtocolEntry, ProtocolEntryTrait},
    executable::middleware::AsyncMiddleware,
    url::UrlRoot,
};

/// Neutral builder for one protocol entry shared by server and client runtimes.
pub struct ProtocolEntryBuilder<P, TS = crate::connection::tcp::TcpTransport>
where
    P: Protocol<Wire = TS::Wire, Spec = TS> + Clone + 'static,
    TS: TransportSpec,
{
    protocol: Option<P>,
    url: Arc<UrlRoot<P::Context, TS>>,
    middlewares: Vec<Arc<dyn AsyncMiddleware<P::Context>>>,
}

impl<P, TS> ProtocolEntryBuilder<P, TS>
where
    P: Protocol<Wire = TS::Wire, Spec = TS> + Clone + 'static,
    TS: TransportSpec,
{
    pub fn new(protocol: P) -> Self {
        Self {
            protocol: Some(protocol),
            url: Arc::new(UrlRoot::<P::Context, TS>::default()),
            middlewares: Vec::new(),
        }
    }

    pub fn set_url(mut self, url: Arc<UrlRoot<P::Context, TS>>) -> Self {
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

    pub fn build(self) -> Arc<dyn ProtocolEntryTrait<TS>> {
        Arc::new(ProtocolEntry::new(
            self.protocol.expect("Protocol must be set"),
            self.url,
            self.middlewares,
        ))
    }
}
