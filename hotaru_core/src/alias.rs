//! Backward-compatible re-export of [`crate::marker`].
//!
//! New code should import from `hotaru_core::marker`; this module remains so
//! older users of `hotaru_core::alias::*` keep compiling during the rename.

pub use crate::marker::*;
