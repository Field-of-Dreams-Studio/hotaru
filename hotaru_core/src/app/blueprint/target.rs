//! TEMPORARY (0.8) — App-target → Blueprint storage-group bridge.
//!
//! This module is deliberately **not** part of the long-term design. It exists
//! only so 0.8 can turn an [`AppTarget`] marker into the concrete homogeneous
//! storage groups a [`Blueprint`](super::Blueprint) needs. The conceptual
//! Blueprint axis stays `Blueprint<TS, T: AppTarget>`, which is the intended
//! long-term shape.
//!
//! Why it is temporary: this trait restates the role → flavour table that
//! `Accepts<H>` already owns at bind time (`InboundOnly` → endpoint,
//! `OutboundOnly` → outpoint, `Both` → both). Keeping a second copy of that
//! table is acceptable as a short-lived 0.8 bridge, not as the final model.
//!
//! 0.9 replaces this bridge; see `hotaru_core/blueprint_0_9_update_plan.md`.
//! Do not grow new responsibilities here — extend the 0.9 plan instead.

use crate::app::{AppTarget, Both, InboundOnly, OutboundOnly};
use crate::connection::TransportSpec;
use crate::executable::def::{EndpointHandler, OutpointHandler};
use crate::prelude::{Arc, Vec, vec};
use crate::protocol::Protocol;

use super::{ErasedHomoBlueprint, HomoBlueprint, ProtocolDef};

/// TEMP (0.8): builds the erased storage groups allowed by an App target.
///
/// `InboundOnly` yields one endpoint group, `OutboundOnly` one outpoint group,
/// and `Both` two groups sharing the same `Arc<ProtocolDef<P>>`. This is
/// crate-private 0.8 plumbing; the 0.9 redesign removes it (see
/// `blueprint_0_9_update_plan.md`).
pub(crate) trait TargetGroups: AppTarget {
    fn make_groups<TS, P>(def: Arc<ProtocolDef<P>>) -> Vec<ErasedHomoBlueprint<TS>>
    where
        TS: TransportSpec,
        P: Protocol<TS = TS, Wire = TS::Wire>;
}

impl TargetGroups for InboundOnly {
    fn make_groups<TS, P>(def: Arc<ProtocolDef<P>>) -> Vec<ErasedHomoBlueprint<TS>>
    where
        TS: TransportSpec,
        P: Protocol<TS = TS, Wire = TS::Wire>,
    {
        vec![Arc::new(HomoBlueprint::<P, EndpointHandler<P>>::from_def(
            def,
        ))]
    }
}

impl TargetGroups for OutboundOnly {
    fn make_groups<TS, P>(def: Arc<ProtocolDef<P>>) -> Vec<ErasedHomoBlueprint<TS>>
    where
        TS: TransportSpec,
        P: Protocol<TS = TS, Wire = TS::Wire>,
    {
        vec![Arc::new(HomoBlueprint::<P, OutpointHandler<P>>::from_def(
            def,
        ))]
    }
}

impl TargetGroups for Both {
    fn make_groups<TS, P>(def: Arc<ProtocolDef<P>>) -> Vec<ErasedHomoBlueprint<TS>>
    where
        TS: TransportSpec,
        P: Protocol<TS = TS, Wire = TS::Wire>,
    {
        vec![
            Arc::new(HomoBlueprint::<P, EndpointHandler<P>>::from_def(
                def.clone(),
            )),
            Arc::new(HomoBlueprint::<P, OutpointHandler<P>>::from_def(def)),
        ]
    }
}
