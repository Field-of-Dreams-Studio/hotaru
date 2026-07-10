//! Embassy runtime backend for Hotaru.
//!
//! The default feature set is `no_std + alloc`: `embedded` selects
//! `hotaru_core/embedded`, and `spawn_local` matches Embassy's usual
//! single-executor task model.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(all(feature = "std", feature = "embedded"))]
compile_error!("hotaru_rt_embassy: features `std` and `embedded` are mutually exclusive");

#[cfg(not(any(feature = "std", feature = "embedded")))]
compile_error!("hotaru_rt_embassy: enable exactly one of `std` or `embedded`");

#[cfg(all(feature = "spawn_send", feature = "spawn_local"))]
compile_error!("hotaru_rt_embassy: features `spawn_send` and `spawn_local` are mutually exclusive");

#[cfg(not(any(feature = "spawn_send", feature = "spawn_local")))]
compile_error!("hotaru_rt_embassy: enable exactly one of `spawn_send` or `spawn_local`");

mod mutex;
mod once_cell;
mod runtime;

pub use mutex::{EmbassyMutex, EmbassyRawMutex};
pub use once_cell::EmbassyOnceCell;
pub use runtime::{EmbassyJoinError, EmbassyJoinHandle, EmbassyTimeoutError};

#[doc(hidden)]
pub mod __private {
    pub use embassy_executor;
    pub use embassy_time;
    pub use hotaru_core;

    pub use crate::{
        EmbassyMutex, EmbassyOnceCell, EmbassyRawMutex,
        runtime::{
            EmbassyJobQueue, EmbassyRuntimeState, run_queued_jobs, select2, spawn_join,
            spawn_join_with_mutex, spawn_task, spawn_task_with_mutex, to_embassy_duration,
        },
    };
}
