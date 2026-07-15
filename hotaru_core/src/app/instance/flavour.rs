//! Compile-time role/flavour safety checks. No runtime logic.

use crate::{
    executable::def::{EndpointHandler, OutpointHandler},
    protocol::Protocol,
};

use super::target::{AppTarget, Both, InboundOnly, OutboundOnly};

mod sealed {
    pub trait Sealed {}
}

/// Allows only role/flavour pairs defined by this crate.
pub trait Accepts<H>: AppTarget + sealed::Sealed {}

impl sealed::Sealed for InboundOnly {}
impl sealed::Sealed for OutboundOnly {}
impl sealed::Sealed for Both {}

impl<P: Protocol> Accepts<EndpointHandler<P>> for InboundOnly {}
impl<P: Protocol> Accepts<OutpointHandler<P>> for OutboundOnly {}
impl<P: Protocol> Accepts<EndpointHandler<P>> for Both {}
impl<P: Protocol> Accepts<OutpointHandler<P>> for Both {}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_accepts<T, H>()
    where
        T: Accepts<H>,
    {
    }

    // Generic bodies are type-checked even without constructing a concrete
    // protocol, so this pins all four positive role/flavour combinations.
    #[allow(dead_code)]
    fn positive_acceptance_matrix<P: Protocol>() {
        assert_accepts::<InboundOnly, EndpointHandler<P>>();
        assert_accepts::<OutboundOnly, OutpointHandler<P>>();
        assert_accepts::<Both, EndpointHandler<P>>();
        assert_accepts::<Both, OutpointHandler<P>>();
    }
}
