use crate::executable::def::{AccessPointDef, FinalHandlerDef};
use crate::prelude::Vec;
use crate::protocol::Protocol;

/// All retained routes for one concrete protocol and one AP flavour.
pub struct AccessPoints<P: Protocol, H: FinalHandlerDef<P>> {
    pub(crate) defs: Vec<AccessPointDef<P, H>>,
}
