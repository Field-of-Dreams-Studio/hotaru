use crate::executable::middleware::{AsyncMiddleware, AsyncMiddlewareChain};
use crate::prelude::{Arc, Vec};
use crate::protocol::RequestContext;

/// One slot in the user's middleware chain. `Inherit` is expanded to
/// the protocol root chain at preparation time.
pub enum MiddlewareSlot<C: RequestContext> {
    Concrete(Arc<dyn AsyncMiddleware<C>>),
    Inherit,
}

/// Owned middleware-slot collection for an access-point definition.
///
/// Symbolic `Inherit` entries remain unresolved until this collection is
/// consumed into the concrete chain used by an executable binding.
pub(crate) struct MiddlewareSlots<C: RequestContext>(Vec<MiddlewareSlot<C>>);

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

impl<C: RequestContext> MiddlewareSlots<C> {
    pub(crate) fn new(slots: Vec<MiddlewareSlot<C>>) -> Self {
        Self(slots)
    }

    pub(crate) fn inheriting() -> Self {
        let mut slots = Vec::with_capacity(1);
        slots.push(MiddlewareSlot::Inherit);
        Self(slots)
    }

    pub(crate) fn push(&mut self, slot: MiddlewareSlot<C>) {
        self.0.push(slot);
    }

    pub(crate) fn remove_inherit(&mut self) {
        self.0
            .retain(|slot| !matches!(slot, MiddlewareSlot::Inherit));
    }

    pub(crate) fn as_slice(&self) -> &[MiddlewareSlot<C>] {
        &self.0
    }

    /// Consume these slots and resolve every `Inherit` entry against one
    /// captured root-middleware snapshot. An optional flavour-specific
    /// middleware is always prepended.
    pub(crate) fn into_chain(
        self,
        inherited: &[Arc<dyn AsyncMiddleware<C>>],
        prefix: Option<Arc<dyn AsyncMiddleware<C>>>,
    ) -> AsyncMiddlewareChain<C> {
        let mut chain: AsyncMiddlewareChain<C> = AsyncMiddlewareChain::new();

        if let Some(prefix) = prefix {
            chain.push(prefix);
        }

        for slot in self.0 {
            match slot {
                MiddlewareSlot::Concrete(middleware) => chain.push(middleware),
                MiddlewareSlot::Inherit => chain.extend(inherited.iter().cloned()),
            }
        }

        chain
    }
}
