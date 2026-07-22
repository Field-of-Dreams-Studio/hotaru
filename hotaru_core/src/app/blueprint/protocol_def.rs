use crate::executable::middleware::AsyncMiddlewareChain;
use crate::protocol::Protocol;

/// Repeatable construction data for exactly one protocol entry.
pub struct ProtocolDef<P: Protocol> {
    pub(crate) protocol: P,
    pub(crate) root_middlewares: AsyncMiddlewareChain<P::Context>,
}
