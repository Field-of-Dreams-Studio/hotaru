//! Debug logging module for development-time diagnostics
//! 
//! This module provides conditional compilation macros that enable detailed logging
//! during development while ensuring zero runtime overhead in production builds.
//! All macros are controlled by the `dev-log` feature flag.
//! 
//! # Features
//! 
//! - **Zero-cost abstraction**: Completely removed when `dev-log` is disabled
//! - **Multiple log levels**: debug, error, warn, and trace
//! - **Value inspection**: Similar to `dbg!` but feature-gated
//! - **Type-safe formatting**: All macros use standard Rust formatting
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
//! 
//! debug_log!("Server started on port {}", 8080);
//! debug_error!("Connection failed: {}", err);
//! ```

/// General-purpose debug logging macro
/// 
/// Outputs informational messages prefixed with `[DEBUG]`.
/// Use for general application state and flow information.
/// 
/// # Examples
/// ```rust
/// use hotaru_core::debug_log;
/// debug_log!("Connection established");
/// debug_log!("Processing {} requests", count);
/// ```
#[macro_export]
#[cfg(feature = "dev-log")]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        println!("[DEBUG] {}", format!($($arg)*));
    };
}

#[macro_export]
#[cfg(not(feature = "dev-log"))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}

/// Error logging macro for recoverable errors
/// 
/// Outputs to stderr with `[ERROR]` prefix.
/// Use for errors that don't terminate the application.
/// 
/// # Examples
/// ```rust
/// use hotaru_core::debug_error;
/// debug_error!("Failed to parse header: {}", e);
/// debug_error!("Invalid configuration: {:?}", config);
/// ```
#[macro_export]
#[cfg(feature = "dev-log")]
macro_rules! debug_error {
    ($($arg:tt)*) => {
        eprintln!("[ERROR] {}", format!($($arg)*));
    };
}

#[macro_export]
#[cfg(not(feature = "dev-log"))]
macro_rules! debug_error {
    ($($arg:tt)*) => {};
}

/// Warning logging macro for potentially problematic conditions
/// 
/// Outputs to stderr with `[WARN]` prefix.
/// Use for deprecations, performance issues, or recoverable problems.
/// 
/// # Examples
/// ```rust
/// use hotaru_core::debug_warn;
/// debug_warn!("Connection timeout after {:?}", duration);
/// debug_warn!("Using deprecated API");
/// ```
#[macro_export]
#[cfg(feature = "dev-log")]
macro_rules! debug_warn {
    ($($arg:tt)*) => {
        eprintln!("[WARN] {}", format!($($arg)*));
    };
}

#[macro_export]
#[cfg(not(feature = "dev-log"))]
macro_rules! debug_warn {
    ($($arg:tt)*) => {};
}

/// Detailed trace logging for verbose debugging
/// 
/// Outputs with `[TRACE]` prefix.
/// Use for detailed execution flow and state transitions.
/// 
/// # Examples
/// ```rust
/// use hotaru_core::debug_trace;
/// debug_trace!("Entering function: {}", fn_name);
/// debug_trace!("Loop iteration {}: state={:?}", i, state);
/// ```
#[macro_export]
#[cfg(feature = "dev-log")]
macro_rules! debug_trace {
    ($($arg:tt)*) => {
        println!("[TRACE] {}", format!($($arg)*));
    };
}

#[macro_export]
#[cfg(not(feature = "dev-log"))]
macro_rules! debug_trace {
    ($($arg:tt)*) => {};
}

/// Value inspection macro similar to `dbg!`
/// 
/// Prints file location and expression values.
/// Returns the value, making it usable in expressions.
/// 
/// # Examples
/// ```rust
/// use hotaru_core::debug_value;
/// let result = debug_value!(calculate());
/// debug_value!(x, y, z);  // Multiple values
/// ```
/// 
/// # Output Format
/// ```
/// [src/main.rs:42] calculate() = 42
/// ```
#[macro_export]
#[cfg(feature = "dev-log")]
macro_rules! debug_value {
    () => {
        eprintln!("[{}:{}]", file!(), line!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                eprintln!("[{}:{}] {} = {:#?}",
                    file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::debug_value!($val)),+,)
    };
}

#[macro_export]
#[cfg(not(feature = "dev-log"))]
macro_rules! debug_value {
    () => {};
    ($val:expr $(,)?) => { $val };
    ($($val:expr),+ $(,)?) => { ($($val),+,) };
}