//! Debug logging module for development-time diagnostics.
//!
//! Provides conditional compilation macros that enable detailed logging
//! during development while ensuring zero runtime overhead in production
//! builds. All macros are controlled by the `dev-log` feature flag.
//!
//! # Three-arm design (`dev-log` × `std` / `embedded`)
//!
//! Each macro has three cfg-arms so no build ever pulls in std machinery
//! it doesn't want:
//!
//! | `dev-log` | `std` / `embedded` | Expansion |
//! |---|---|---|
//! | off | either | *nothing* — arguments are not evaluated. |
//! | on  | `std`  | `println!` / `eprintln!` with `[LEVEL]` prefix (the classic behaviour). |
//! | on  | `embedded` | **Hollow expansion** — `let _ = core::format_args!(...)`. Arguments are still evaluated so `unused_variables` warnings at call sites stay quiet, but no formatting, allocation, or IO happens. |
//!
//! The hollow embedded arm exists because embedded builds are `#![no_std]`
//! — `println!` / `eprintln!` are gone, and `format!` allocates via
//! `alloc` (which is available but wasteful for a no-op). `format_args!`
//! is `core::` and creates a stack-only `core::fmt::Arguments` value at
//! zero allocation cost. Wrapping in `let _ = ...;` marks the resulting
//! value as intentionally discarded.
//!
//! # Future: hooking a real embedded logger
//!
//! When HCR picks an embedded logger (defmt / semihosting / RTT / the
//! `log` crate), route the hollow arm through a user-installed sink
//! function — the macro body changes, call sites don't. Design sketch:
//! a `static LOGGER: OnceCell<fn(core::fmt::Arguments<'_>)>` that the
//! user's `#[entry]` installs once, and the embedded arm calls it if
//! present. Deferred until an actual logger is chosen.
//!
//! # Usage
//!
//! Enable the feature in your Cargo.toml or via command line:
//! ```bash
//! cargo run --features "dev-log"
//! ```
//!
//! Then import and use the macros:
//! ```rust
//! use hotaru_core::{debug_log, debug_error};
//! # let err = "unreachable";
//! debug_log!("Server started on port {}", 8080);
//! debug_error!("Connection failed: {}", err);
//! ```

// ---------------------------------------------------------------------------
// debug_log! — general-purpose info logging (stdout under std)
// ---------------------------------------------------------------------------

/// General-purpose debug logging macro.
///
/// See the module docs for the three-arm expansion rules.
///
/// # Examples
/// ```rust
/// use hotaru_core::debug_log;
/// # let count = 0;
/// debug_log!("Connection established");
/// debug_log!("Processing {} requests", count);
/// ```
#[macro_export]
#[cfg(all(feature = "dev-log", feature = "std"))]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        ::std::println!("[DEBUG] {}", ::std::format!($($arg)*));
    };
}

/// `dev-log + embedded` — hollow expansion. See the module docs for
/// why we use `core::format_args!` here instead of a full no-op.
#[macro_export]
#[cfg(all(feature = "dev-log", feature = "embedded"))]
macro_rules! debug_log {
    ($($arg:tt)*) => {{
        let _ = ::core::format_args!($($arg)*);
    }};
}

#[macro_export]
#[cfg(not(feature = "dev-log"))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}

// ---------------------------------------------------------------------------
// debug_error! — recoverable errors (stderr under std)
// ---------------------------------------------------------------------------

/// Error logging macro for recoverable errors.
///
/// See the module docs for the three-arm expansion rules.
///
/// # Examples
/// ```rust
/// # let e = "boom";
/// # #[derive(Debug)] struct Config;
/// # let config = Config;
/// use hotaru_core::debug_error;
/// debug_error!("Failed to parse header: {}", e);
/// debug_error!("Invalid configuration: {:?}", config);
/// ```
#[macro_export]
#[cfg(all(feature = "dev-log", feature = "std"))]
macro_rules! debug_error {
    ($($arg:tt)*) => {
        ::std::eprintln!("[ERROR] {}", ::std::format!($($arg)*));
    };
}

/// `dev-log + embedded` — hollow expansion.
#[macro_export]
#[cfg(all(feature = "dev-log", feature = "embedded"))]
macro_rules! debug_error {
    ($($arg:tt)*) => {{
        let _ = ::core::format_args!($($arg)*);
    }};
}

#[macro_export]
#[cfg(not(feature = "dev-log"))]
macro_rules! debug_error {
    ($($arg:tt)*) => {};
}

// ---------------------------------------------------------------------------
// debug_warn! — recoverable warnings (stderr under std)
// ---------------------------------------------------------------------------

/// Warning logging macro for potentially problematic conditions.
///
/// See the module docs for the three-arm expansion rules.
///
/// # Examples
/// ```rust
/// # let duration = std::time::Duration::from_secs(1);
/// use hotaru_core::debug_warn;
/// debug_warn!("Connection timeout after {:?}", duration);
/// debug_warn!("Using deprecated API");
/// ```
#[macro_export]
#[cfg(all(feature = "dev-log", feature = "std"))]
macro_rules! debug_warn {
    ($($arg:tt)*) => {
        ::std::eprintln!("[WARN] {}", ::std::format!($($arg)*));
    };
}

/// `dev-log + embedded` — hollow expansion.
#[macro_export]
#[cfg(all(feature = "dev-log", feature = "embedded"))]
macro_rules! debug_warn {
    ($($arg:tt)*) => {{
        let _ = ::core::format_args!($($arg)*);
    }};
}

#[macro_export]
#[cfg(not(feature = "dev-log"))]
macro_rules! debug_warn {
    ($($arg:tt)*) => {};
}

// ---------------------------------------------------------------------------
// debug_trace! — verbose tracing (stdout under std)
// ---------------------------------------------------------------------------

/// Detailed trace logging for verbose debugging.
///
/// See the module docs for the three-arm expansion rules.
///
/// # Examples
/// ```rust
/// # let fn_name = "";
/// # let (i, state) = (0, ());
/// use hotaru_core::debug_trace;
/// debug_trace!("Entering function: {}", fn_name);
/// debug_trace!("Loop iteration {}: state={:?}", i, state);
/// ```
#[macro_export]
#[cfg(all(feature = "dev-log", feature = "std"))]
macro_rules! debug_trace {
    ($($arg:tt)*) => {
        ::std::println!("[TRACE] {}", ::std::format!($($arg)*));
    };
}

/// `dev-log + embedded` — hollow expansion.
#[macro_export]
#[cfg(all(feature = "dev-log", feature = "embedded"))]
macro_rules! debug_trace {
    ($($arg:tt)*) => {{
        let _ = ::core::format_args!($($arg)*);
    }};
}

#[macro_export]
#[cfg(not(feature = "dev-log"))]
macro_rules! debug_trace {
    ($($arg:tt)*) => {};
}

// ---------------------------------------------------------------------------
// debug_value! — dbg!-style value inspection
// ---------------------------------------------------------------------------

/// Value inspection macro similar to `dbg!`.
///
/// Under `dev-log + std`, prints file location and expression values to
/// stderr. Under `dev-log + embedded` and under `not(dev-log)`, the
/// expression evaluates to itself with no output — semantic identity is
/// preserved so `let x = debug_value!(compute());` keeps working.
///
/// # Examples
/// ```rust
/// # fn calculate() -> i32 { 42 }
/// use hotaru_core::debug_value;
/// let result = debug_value!(calculate());
/// ```
///
/// # Output format (`dev-log + std` only)
/// ```text
/// [src/main.rs:42] calculate() = 42
/// ```
#[macro_export]
#[cfg(all(feature = "dev-log", feature = "std"))]
macro_rules! debug_value {
    () => {
        ::std::eprintln!("[{}:{}]", file!(), line!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                ::std::eprintln!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::debug_value!($val)),+,)
    };
}

/// `dev-log + embedded` — identity on values, no-op on the marker form.
/// No `Debug` bound is required so this compiles for any `T`.
#[macro_export]
#[cfg(all(feature = "dev-log", feature = "embedded"))]
macro_rules! debug_value {
    () => {};
    ($val:expr $(,)?) => { $val };
    ($($val:expr),+ $(,)?) => { ($($val),+,) };
}

#[macro_export]
#[cfg(not(feature = "dev-log"))]
macro_rules! debug_value {
    () => {};
    ($val:expr $(,)?) => { $val };
    ($($val:expr),+ $(,)?) => { ($($val),+,) };
}
