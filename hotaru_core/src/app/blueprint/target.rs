use crate::app::AppTarget;
use crate::connection::TransportSpec;
use crate::prelude::{Arc, Vec};
use crate::protocol::Protocol;

use super::{ErasedHomoBlueprint, ProtocolDef};

/// Creates only the flavour groups allowed by an App target.
pub(crate) trait TargetGroups: AppTarget {
    fn make_groups<TS, P>(def: Arc<ProtocolDef<P>>) -> Vec<ErasedHomoBlueprint<TS>>
    where
        TS: TransportSpec,
        P: Protocol<TS = TS, Wire = TS::Wire>;
}
