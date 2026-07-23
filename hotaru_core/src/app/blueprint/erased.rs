use core::any::{Any, TypeId};

use crate::app::registry::ProtocolRegistryKind;
use crate::connection::TransportSpec;
use crate::executable::registry::ProtocolEntryRegistry;
use crate::marker::MaybeSendSync;
use crate::prelude::Arc;

use super::BlueprintError;

/// Private object-safe boundary for erased protocol/flavour groups.
///
/// The erasure axis is `(P, H)`, never the App target `T`. Admission uses
/// `protocol_type_id` / `as_any` to reject duplicates and downcast to the
/// concrete group; application uses `materialize_into` (builder path, creates
/// the entry if missing) and `register_into` (built-App path, requires the
/// entry to already exist).
pub(crate) trait HomoBluePrintTrait<TS: TransportSpec>: MaybeSendSync {
    fn protocol_type_id(&self) -> TypeId;
    fn protocol_name(&self) -> &'static str;
    fn flavour(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
    fn len(&self) -> usize;
    fn materialize_into(
        &self,
        registry: &mut ProtocolEntryRegistry<TS>,
    ) -> Result<(), BlueprintError>;
    fn has_entry(&self, registry: &ProtocolRegistryKind<TS>) -> bool;
    fn register_into(&self, registry: &ProtocolRegistryKind<TS>) -> Result<(), BlueprintError>;
}

pub(crate) type ErasedHomoBlueprint<TS> = Arc<dyn HomoBluePrintTrait<TS>>;
