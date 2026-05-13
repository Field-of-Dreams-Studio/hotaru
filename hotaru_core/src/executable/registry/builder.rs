use std::sync::Arc;

use crate::{
    connection::TransportSpec,
    executable::{
        ProtocolEntryBuilder, entry::ProtocolEntryTrait, registry::ProtocolEntryRegistry,
    },
    protocol::Protocol,
};

/// Builder for protocol registries assembled from neutral protocol entries.
pub struct ProtocolRegistryBuilder<TS: TransportSpec> {
    handlers: Vec<Arc<dyn ProtocolEntryTrait<TS>>>,
}

impl<TS: TransportSpec> ProtocolRegistryBuilder<TS> {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn protocol<P: Protocol<Wire = TS::Wire, TS = TS> + Clone + 'static>(
        mut self,
        builder: ProtocolEntryBuilder<P, TS>,
    ) -> Self {
        self.handlers.push(builder.build());
        self
    }

    pub fn build(self) -> ProtocolEntryRegistry<TS> {
        ProtocolEntryRegistry {
            handlers: self.handlers,
        }
    }
}
