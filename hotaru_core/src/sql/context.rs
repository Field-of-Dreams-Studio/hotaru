//! SQL context for request handling

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::connection::{ProtocolRole, RequestContext};
use crate::extensions::{Locals, Params};
use crate::sql::{SqlClient, SqlResult};
use crate::url::PathPattern;

/// SQL context for query execution
///
/// This context flows through handlers and middleware for SQL operations.
/// It follows the same pattern as HttpContext but for database queries.
pub struct Context {
    /// The SQL query string
    pub query: String,
    /// Query parameters
    pub params: Vec<akari::Value>,
    /// Query response (populated after execution)
    pub response: SqlResult,

    // Outpoint-specific fields (same as HttpContext pattern)
    /// Reference to the SqlClient
    pub client: Option<Arc<SqlClient>>,
    /// Query path patterns (e.g., /users/<int:id>)
    pub query_patterns: Vec<PathPattern>,
    /// Named query parameters
    pub query_names: Vec<Option<String>>,
    /// Query parameter values
    pub query_params: HashMap<String, String>,

    // Shared middleware fields
    /// Configuration parameters
    pub config: Params,
    /// Local variables
    pub locals: Locals,

    /// Executable marker (currently unused for SQL)
    pub executable: SqlExecutable,
}

/// Executable context marker for SQL
#[derive(Debug, Clone)]
pub enum SqlExecutable {
    /// Query execution
    Query,
}

impl Context {
    /// Create a new SQL context
    pub fn new() -> Self {
        Self {
            query: String::new(),
            params: Vec::new(),
            response: SqlResult::new(),
            client: None,
            query_patterns: Vec::new(),
            query_names: Vec::new(),
            query_params: HashMap::new(),
            config: Params::new(),
            locals: Locals::new(),
            executable: SqlExecutable::Query,
        }
    }

    /// Create a new SQL context with client
    pub fn new_with_client(client: Arc<SqlClient>) -> Self {
        Self {
            query: String::new(),
            params: Vec::new(),
            response: SqlResult::new(),
            client: Some(client),
            query_patterns: Vec::new(),
            query_names: Vec::new(),
            query_params: HashMap::new(),
            config: Params::new(),
            locals: Locals::new(),
            executable: SqlExecutable::Query,
        }
    }

    /// Set query patterns with builder pattern
    pub fn with_query_patterns(
        mut self,
        patterns: Vec<PathPattern>,
        names: Vec<Option<String>>,
    ) -> Self {
        self.query_patterns = patterns;
        self.query_names = names;
        self
    }

    /// Set query patterns mutably
    pub fn set_query_patterns(
        &mut self,
        patterns: Vec<PathPattern>,
        names: Vec<Option<String>>,
    ) -> &mut Self {
        self.query_patterns = patterns;
        self.query_names = names;
        self
    }

    /// Add a query parameter (builder pattern)
    pub fn with_param<K: Into<String>, V: Into<String>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.query_params.insert(key.into(), value.into());
        self
    }

    /// Add a query parameter mutably
    pub fn add_param<K: Into<String>, V: Into<String>>(
        &mut self,
        key: K,
        value: V,
    ) -> &mut Self {
        self.query_params.insert(key.into(), value.into());
        self
    }

    /// Set config value with builder pattern
    pub fn set_config<V: Send + Sync + 'static>(mut self, value: V) -> Self {
        self.config.set(value);
        self
    }

    /// Get config value
    pub fn get_config<V: Clone + Send + Sync + 'static>(&self) -> Option<V> {
        self.config.get::<V>().cloned()
    }

    /// Set local value with builder pattern
    pub fn set_local<K: Into<String>, V: Send + Sync + 'static>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        let key = key.into();
        if let Some(s) = (&value as &dyn Any).downcast_ref::<&'static str>().copied() {
            self.locals.set(key.clone(), s);
            self.locals.set(key, s.to_string());
        } else {
            self.locals.set(key, value);
        }
        self
    }

    /// Get local value
    pub fn get_local<V: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<V> {
        self.locals.get::<V>(key).cloned()
    }

    /// Get a named parameter from the query pattern
    /// For example, with pattern "/users/<int:id>", param("id") returns the value
    pub fn param(&self, name: &str) -> Option<String> {
        self.query_params.get(name).cloned()
    }

    /// Get the database connection pool from the client
    ///
    /// Returns the type-erased pool which can be downcast to the specific type:
    /// - `sqlx::Pool<sqlx::Postgres>` for PostgreSQL
    /// - `sqlx::Pool<sqlx::MySql>` for MySQL
    /// - `sqlx::Pool<sqlx::Sqlite>` for SQLite
    #[cfg(feature = "sql")]
    pub async fn pool<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        use crate::sql::pool::{ConnectionPool, ConnectionKey};

        let client = self.client.as_ref()?;
        let key = ConnectionKey {
            database_url: client.connection_string.clone(),
            db_type: client.db_type.clone(),
        };

        let pool = ConnectionPool::global().get(&key).await?;
        pool.downcast::<T>().ok()
    }

    /// Get the database connection pool from the client (non-async version for handler convenience)
    ///
    /// Note: This uses tokio::runtime::Handle::current() to run the async operation.
    /// Only use this inside async contexts.
    #[cfg(feature = "sql")]
    pub fn pool_sync<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.pool::<T>().await
            })
        })
    }

    /// Get client reference
    pub fn client(&self) -> Option<&Arc<SqlClient>> {
        self.client.as_ref()
    }

    /// Set the query string
    pub fn set_query(&mut self, query: impl Into<String>) -> &mut Self {
        self.query = query.into();
        self
    }

    /// Add a parameter value
    pub fn add_query_param(&mut self, value: akari::Value) -> &mut Self {
        self.params.push(value);
        self
    }

    /// Set the result
    pub fn set_result(&mut self, result: SqlResult) -> &mut Self {
        self.response = result;
        self
    }

    /// Get the result
    pub fn result(&self) -> &SqlResult {
        &self.response
    }

    /// Take the result (consumes it)
    pub fn take_result(&mut self) -> SqlResult {
        std::mem::take(&mut self.response)
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

impl RequestContext for Context {
    type Request = String;
    type Response = SqlResult;

    fn handle_error(&mut self) {
        // Set empty result on error
        self.response = SqlResult::new();
    }

    fn role(&self) -> ProtocolRole {
        ProtocolRole::Client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = Context::new();
        assert!(ctx.query.is_empty());
        assert!(ctx.response.is_empty());
        assert!(ctx.client.is_none());
    }

    #[test]
    fn test_context_params() {
        let mut ctx = Context::new();
        ctx.add_param("id", "123");
        assert_eq!(ctx.param("id"), Some("123".to_string()));
    }

    #[test]
    fn test_context_builder() {
        let ctx = Context::new()
            .with_param("user_id", "456")
            .set_local("cache_key", "user:456".to_string());

        assert_eq!(ctx.param("user_id"), Some("456".to_string()));
        assert_eq!(ctx.get_local::<String>("cache_key"), Some("user:456".to_string()));
    }
}
