use crate::connection::TransportSpec;
use crate::executable::def::FinalHandlerDef;
use crate::marker::MaybeSendSync;
use crate::prelude::{Arc, PRwLock};
use crate::protocol::Protocol;

use super::{AccessPoints, ProtocolDef};

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

/// Private object-safe boundary for erased protocol/flavour groups.
///
/// Its behavior is intentionally added in a later segment.
pub(crate) trait HomoBluePrintTrait<TS: TransportSpec>: MaybeSendSync {}

pub(crate) type ErasedHomoBlueprint<TS> = Arc<dyn HomoBluePrintTrait<TS>>;
