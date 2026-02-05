//! SQL outpoint layer for Hotaru
//!
//! This module provides a SQL client/outpoint system that mirrors the HTTP Client pattern.
//! SQL queries are registered as named outpoints (like `/users/by_id`) and can be called
//! from HTTP endpoints or other handlers.
//!
//! # Architecture
//!
//! - **SqlClient** - Like HTTP Client but for database connections
//! - **Outpoints** - Named SQL queries (e.g., `/users/by_id`)
//! - **ClientRegistry** - Reuse existing registry for SQL outpoints
//! - **outpoint! macro** - Same macro used for HTTP Client
//!
//! # Example
//!
//! ```ignore
//! use hotaru::sql::{SqlClient, SSqlClient};
//!
//! pub static DB: SSqlClient = Lazy::new(|| {
//!     SqlClient::postgres("postgres://localhost/mydb")
//!         .name("main_db")
//!         .build()
//! });
//!
//! outpoint! {
//!     DB.query("/users/<int:id>"),
//!     pub get_user<SQL> {
//!         let id: i64 = req.param("id")?.parse()?;
//!         let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
//!             .bind(id)
//!             .fetch_one(&req.pool::<PgPool>()?)
//!             .await?;
//!         SqlResult::single(user)
//!     }
//! }
//! ```

pub mod client;
pub mod context;
pub mod message;
pub mod pool;
pub mod protocol;
pub mod transport;

#[cfg(feature = "sql")]
pub mod middleware;

pub use client::{Client as SqlClient, ClientBuilder as SqlClientBuilder, SClient as SSqlClient};
pub use context::Context as SqlContext;
pub use message::{Message as SqlMessage, QueryResult as SqlResult};
pub use pool::{ConnectionKey as SqlConnectionKey, ConnectionPool as SqlConnectionPool, PoolConfig as SqlPoolConfig, start_cleanup_task as start_sql_cleanup_task};
pub use protocol::SQL;
pub use transport::Transport as SqlTransport;

#[cfg(feature = "sql")]
pub use middleware::*;

/// Database type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DatabaseType {
    Postgres,
    MySQL,
    SQLite,
}
