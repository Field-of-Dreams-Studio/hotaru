use core::sync::atomic::{AtomicU64, Ordering};

use proc_macro::{Ident, Span};

static NEXT_ANONYMOUS_ID: AtomicU64 = AtomicU64::new(0);

/// Returns a process-unique internal name for an anonymous AP.
pub(crate) fn next_anonymous_ident() -> Ident {
    let id = NEXT_ANONYMOUS_ID.fetch_add(1, Ordering::Relaxed);
    Ident::new(&format!("auto_generated_{id}"), Span::call_site())
}
