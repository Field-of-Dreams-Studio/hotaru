//! Compile-time conditional `Send` bound.
//!
//! - With `std`, this is an alias for [`core::marker::Send`].
//! - With `embedded`, this is a no-op marker implemented for all types.
//!
//! Use this on futures that must be `Send` on Tokio/std targets but may be
//! local on embedded/single-threaded targets.

/// Backend-tagged IO adapter.
///
/// The `Backend` parameter makes each adapter a distinct concrete `Self`
/// type, so non-Tokio IO ecosystems can implement Hotaru IO traits without
/// adding broad blanket impls that overlap with Tokio's compatibility blanket.
pub struct IoCompat<T, Backend> {
    inner: T,
    _backend: core::marker::PhantomData<fn() -> Backend>,
}

impl<T, Backend> IoCompat<T, Backend> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _backend: core::marker::PhantomData,
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

/// Alias for `Send` on `std` targets, no-op on `embedded` targets. 
#[cfg(feature = "std")]
pub use core::marker::Send as MaybeSend;

/// MaybeSend. The `Send` bound is only required on `std` targets. 
/// For `no_std` targets, this is a no-op marker implemented for all types. 
#[cfg(feature = "embedded")]
pub trait MaybeSend {}
#[cfg(feature = "embedded")]
impl<T: ?Sized> MaybeSend for T {} 
