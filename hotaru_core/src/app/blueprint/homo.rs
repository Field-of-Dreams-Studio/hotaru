use core::any::{Any, TypeId, type_name};

use crate::app::registry::ProtocolRegistryKind;
use crate::connection::TransportSpec;
use crate::executable::def::FinalHandlerDef;
use crate::executable::registry::ProtocolEntryRegistry;
use crate::marker::MaybeSendSync;
use crate::prelude::{Arc, PRwLock};
use crate::protocol::Protocol;

use super::{AccessPoints, BlueprintError, HomoBluePrintTrait, ProtocolDef};

/// One concrete protocol/flavour group.
pub struct HomoBlueprint<P: Protocol, H: FinalHandlerDef<P>> {
    pub(crate) protocol: Arc<ProtocolDef<P>>,
    pub(crate) access_points: PRwLock<AccessPoints<P, H>>,
}

impl<P: Protocol, H: FinalHandlerDef<P>> HomoBlueprint<P, H> {
    pub(crate) fn from_def(protocol: Arc<ProtocolDef<P>>) -> Self {
        Self {
            protocol,
            access_points: PRwLock::new(AccessPoints::new()),
        }
    }

    #[cfg(test)]
    pub(crate) fn protocol_def(&self) -> &Arc<ProtocolDef<P>> {
        &self.protocol
    }
}

impl<TS, P, H> HomoBluePrintTrait<TS> for HomoBlueprint<P, H>
where
    TS: TransportSpec,
    P: Protocol<TS = TS, Wire = TS::Wire>,
    H: FinalHandlerDef<P> + MaybeSendSync,
{
    fn protocol_type_id(&self) -> TypeId {
        TypeId::of::<P>()
    }

    fn protocol_name(&self) -> &'static str {
        self.protocol.protocol.name()
    }

    fn flavour(&self) -> &'static str {
        type_name::<H>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn len(&self) -> usize {
        self.access_points.read().defs.len()
    }

    fn materialize_into(
        &self,
        registry: &mut ProtocolEntryRegistry<TS>,
    ) -> Result<(), BlueprintError> {
        if registry.entry::<P>().is_none() {
            registry.register(self.protocol.as_ref());
        }
        let entry = registry
            .entry::<P>()
            .expect("entry exists: pre-existing or registered just above");
        let points = self.access_points.read();
        for def in points.defs.iter() {
            entry.register(def)?;
        }
        Ok(())
    }

    fn has_entry(&self, registry: &ProtocolRegistryKind<TS>) -> bool {
        registry.entry::<P>().is_some()
    }

    fn register_into(&self, registry: &ProtocolRegistryKind<TS>) -> Result<(), BlueprintError> {
        if registry.entry::<P>().is_none() {
            return Err(BlueprintError::ProtocolNotFound(
                self.protocol.protocol.name(),
            ));
        }
        let points = self.access_points.read();
        for def in points.defs.iter() {
            registry.register(def)?;
        }
        Ok(())
    }
}
