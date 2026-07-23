use crate::executable::def::{AccessPointDef, FinalHandlerDef};
use crate::prelude::Vec;
use crate::protocol::Protocol;

/// All retained routes for one concrete protocol and one AP flavour.
pub struct AccessPoints<P: Protocol, H: FinalHandlerDef<P>> {
    pub(crate) defs: Vec<AccessPointDef<P, H>>,
}

impl<P: Protocol, H: FinalHandlerDef<P>> AccessPoints<P, H> {
    pub fn new() -> Self {
        Self { defs: Vec::new() }
    }

    pub fn defs(&self) -> &[AccessPointDef<P, H>] {
        &self.defs
    }
}

impl<P: Protocol, H: FinalHandlerDef<P>> Default for AccessPoints<P, H> {
    fn default() -> Self {
        Self::new()
    }
}
