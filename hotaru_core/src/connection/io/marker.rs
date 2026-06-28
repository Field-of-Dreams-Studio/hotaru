//! Compile-time conditional `Send` bound.
//!
//! - With `std`, this is an alias for [`core::marker::Send`].
//! - With `embedded`, this is a no-op marker implemented for all types.
//!
//! Use this on futures that must be `Send` on Tokio/std targets but may be
//! local on embedded/single-threaded targets.

/// Alias for `Send` on `std` targets, no-op on `embedded` targets. 
#[cfg(feature = "std")]
pub use core::marker::Send as MaybeSend;

/// MaybeSend. The `Send` bound is only required on `std` targets. 
/// For `no_std` targets, this is a no-op marker implemented for all types. 
#[cfg(feature = "embedded")]
pub trait MaybeSend {}
#[cfg(feature = "embedded")]
impl<T: ?Sized> MaybeSend for T {} 
