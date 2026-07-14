use crate::{executable::middleware::AsyncMiddleware, protocol::RequestContext};
use crate::prelude::Arc; 

/// One slot in the user's middleware chain. `Inherit` is expanded to
/// the protocol root chain at preparation time.
pub enum MiddlewareSlot<C: RequestContext> {
    Concrete(Arc<dyn AsyncMiddleware<C>>),
    Inherit,
}

impl<C: RequestContext> Clone for MiddlewareSlot<C> {
    // Manual: avoid a spurious `C: Clone` bound from the derive.
    fn clone(&self) -> Self {
        match self {
            Self::Concrete(a) => Self::Concrete(a.clone()),
            Self::Inherit => Self::Inherit,
        }
    }
}

impl<C: RequestContext> core::fmt::Debug for MiddlewareSlot<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Concrete(_) => f.write_str("MiddlewareSlot::Concrete(..)"),
            Self::Inherit => f.write_str("MiddlewareSlot::Inherit"),
        }
    }
}

impl<C: RequestContext> MiddlewareSlot<C> {
    /// Convenience constructor: boxes any `AsyncMiddleware` value into
    /// `Concrete(Arc::new(m))`. Emitted by `endpoint!` / `outpoint!`
    /// for `middleware = [X]` clauses so the expansion stays terse.
    pub fn concrete<M>(m: M) -> Self
    where
        M: AsyncMiddleware<C> + 'static,
    {
        Self::Concrete(Arc::new(m))
    }
}
