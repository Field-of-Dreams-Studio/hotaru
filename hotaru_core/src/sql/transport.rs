//! SQL transport layer
//!
//! The transport for SQL is different from TCP-based protocols.
//! SQL connections are managed through connection pools provided by SQLx.

use std::any::Any;
use std::sync::Arc;

/// SQL transport wrapper
///
/// Unlike TCP-based transports, SQL uses connection pools from SQLx.
/// This is a marker type for the SQL protocol.
#[derive(Debug, Clone)]
pub struct Transport {
    /// Database connection pool (type-erased)
    pool: Option<Arc<dyn Any + Send + Sync>>,
    /// Connection ID
    id: i128,
}

impl Transport {
    /// Create a new SQL transport with a connection pool
    pub fn new(pool: Arc<dyn Any + Send + Sync>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i128;
        Self { pool: Some(pool), id }
    }

    /// Create an empty transport (no pool)
    pub fn empty() -> Self {
        Self { pool: None, id: 0 }
    }

    /// Get the connection pool
    pub fn pool(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.pool.clone()
    }

    /// Check if transport has a pool
    pub fn has_pool(&self) -> bool {
        self.pool.is_some()
    }
}

impl Default for Transport {
    fn default() -> Self {
        Self::empty()
    }
}

impl crate::connection::protocol::Transport for Transport {
    fn id(&self) -> i128 {
        self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
