use core::marker::PhantomData;

use crate::app::AppTarget;
use crate::connection::TransportSpec;
use crate::prelude::{Arc, Vec};

use super::ErasedHomoBlueprint;

/// Reusable, cheaply cloneable set of protocol/flavour groups.
pub struct Blueprint<TS: TransportSpec, T: AppTarget> {
    pub(crate) inner: Arc<BlueprintInner<TS, T>>,
}

pub(crate) struct BlueprintInner<TS: TransportSpec, T: AppTarget> {
    pub(crate) groups: Vec<ErasedHomoBlueprint<TS>>,
    pub(crate) _target: PhantomData<fn() -> T>,
}
