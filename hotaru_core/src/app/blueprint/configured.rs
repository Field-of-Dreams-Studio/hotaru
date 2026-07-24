use crate::app::{
    Accepts, AppTarget,
    common::{OperationalConfig, RunMode},
};
use crate::connection::TransportSpec;
use crate::executable::def::{AccessPointDef, FinalHandlerDef};
use crate::marker::MaybeSendSync;
use crate::protocol::Protocol;

use super::{Blueprint, BlueprintError};

/// A Blueprint plus construction defaults.
///
/// Adds no AP storage: admission delegates to the inner Blueprint, and these
/// defaults only fill unset builder fields when Stage 7 applies it.
pub struct ConfiguredBlueprint<TS: TransportSpec, AT: AppTarget> {
    pub(crate) blueprint: Blueprint<TS, AT>,
    pub(crate) mode: Option<RunMode>,
    pub(crate) operational: Option<OperationalConfig>,
}

impl<TS: TransportSpec, AT: AppTarget> ConfiguredBlueprint<TS, AT> {
    pub fn new(blueprint: Blueprint<TS, AT>) -> Self {
        Self {
            blueprint,
            mode: None,
            operational: None,
        }
    }

    pub fn with_mode(mut self, mode: RunMode) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn with_operational(mut self, operational: OperationalConfig) -> Self {
        self.operational = Some(operational);
        self
    }

    pub fn blueprint(&self) -> &Blueprint<TS, AT> {
        &self.blueprint
    }

    pub fn mode(&self) -> Option<&RunMode> {
        self.mode.as_ref()
    }

    pub fn operational(&self) -> Option<&OperationalConfig> {
        self.operational.as_ref()
    }

    pub fn bind<P, H>(
        &self,
        constructor: fn() -> AccessPointDef<P, H>,
    ) -> Result<(), BlueprintError>
    where
        P: Protocol<TS = TS, Wire = TS::Wire>,
        H: FinalHandlerDef<P> + MaybeSendSync,
        AT: Accepts<H>,
    {
        self.blueprint.bind(constructor)
    }

    pub fn insert<P, H>(&self, def: AccessPointDef<P, H>) -> Result<(), BlueprintError>
    where
        P: Protocol<TS = TS, Wire = TS::Wire>,
        H: FinalHandlerDef<P> + MaybeSendSync,
        AT: Accepts<H>,
    {
        self.blueprint.insert(def)
    }

    pub fn extend<P, H, I>(&self, defs: I) -> Result<(), BlueprintError>
    where
        P: Protocol<TS = TS, Wire = TS::Wire>,
        H: FinalHandlerDef<P> + MaybeSendSync,
        AT: Accepts<H>,
        I: IntoIterator<Item = AccessPointDef<P, H>>,
    {
        self.blueprint.extend(defs)
    }
}
