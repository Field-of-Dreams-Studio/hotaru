use crate::executable::middleware::AsyncMiddlewareChain;
use crate::protocol::Protocol;

/// Repeatable construction data for exactly one protocol entry.
///
/// A `ProtocolDef<P>` is the recipe consumed by `ProtocolEntry::from_def` to
/// build one live protocol entry: it clones the concrete protocol and its root
/// middleware chain, then attaches a fresh URL root. It sits beside
/// `AccessPointDef` in the definition layer because both are pre-registration
/// definitions consumed by the runtime entry, not runtime state themselves.
pub struct ProtocolDef<P: Protocol> {
    pub(crate) protocol: P,
    pub(crate) root_middlewares: AsyncMiddlewareChain<P::Context>,
}

impl<P: Protocol> ProtocolDef<P> {
    pub fn new(protocol: P, root_middlewares: AsyncMiddlewareChain<P::Context>) -> Self {
        Self {
            protocol,
            root_middlewares,
        }
    }

    pub fn protocol(&self) -> &P {
        &self.protocol
    }

    pub fn root_middlewares(&self) -> &AsyncMiddlewareChain<P::Context> {
        &self.root_middlewares
    }
}
