// Type aliases (must be declared before other modules that use it)
pub mod alias;

pub mod http;
// pub mod http2;
pub mod app;
pub mod connection;
pub mod client;
pub mod url;
pub mod debug;

#[cfg(feature = "sql")]
pub mod sql; 

pub use akari::*;

// Re-export commonly used type aliases
pub use alias::{PRwLock, PRwLockReadGuard, PRwLockWriteGuard}; 
