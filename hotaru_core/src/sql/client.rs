//! SQL Client implementation
//!
//! SqlClient follows the same pattern as the HTTP Client but for database connections.

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use once_cell::sync::Lazy;

use crate::app::middleware::AsyncMiddleware;
use crate::client::OutpointRegistration;
use crate::connection::Protocol;
use crate::extensions::{Locals, Params};
use crate::sql::DatabaseType;
use crate::url::parser::parse;

/// SQL Client for database operations
///
/// SqlClient is analogous to HTTP Client but for database connections.
/// It registers query outpoints and manages connection pools.
pub struct Client {
    /// Client name (e.g., "main_db")
    pub name: String,
    /// Database connection string
    pub connection_string: String,
    /// Database type (Postgres, MySQL, SQLite)
    pub db_type: DatabaseType,
    /// Configuration parameters
    pub config: Params,
    /// Static values
    pub statics: Locals,
    /// Protocol-specific middlewares
    middlewares: HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>,
}

/// Builder for SqlClient
pub struct ClientBuilder {
    name: Option<String>,
    connection_string: String,
    db_type: DatabaseType,
    config: Params,
    statics: Locals,
    middlewares: HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>,
}

impl ClientBuilder {
    /// Create a new ClientBuilder with connection string and database type
    pub fn new(connection_string: impl Into<String>, db_type: DatabaseType) -> Self {
        Self {
            name: None,
            connection_string: connection_string.into(),
            db_type,
            config: Params::new(),
            statics: Locals::new(),
            middlewares: HashMap::new(),
        }
    }

    /// Set the client name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set config parameters
    pub fn config(mut self, config: Params) -> Self {
        self.config = config;
        self
    }

    /// Set a single config value
    pub fn set_config<V: Send + Sync + 'static>(mut self, value: V) -> Self {
        self.config.set(value);
        self
    }

    /// Set static values
    pub fn statics(mut self, statics: Locals) -> Self {
        self.statics = statics;
        self
    }

    /// Set a single static value
    pub fn set_static<K: Into<String>, V: Send + Sync + 'static>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.statics.set(key, value);
        self
    }

    /// Add middleware for a protocol
    pub fn middleware<P: Protocol + 'static>(
        mut self,
        mw: Arc<dyn AsyncMiddleware<P::Context>>,
    ) -> Self {
        let entry = self
            .middlewares
            .entry(TypeId::of::<P>())
            .or_insert_with(|| Box::new(Vec::<Arc<dyn AsyncMiddleware<P::Context>>>::new()));
        if let Some(vec) = entry.downcast_mut::<Vec<Arc<dyn AsyncMiddleware<P::Context>>>>() {
            vec.push(mw);
        }
        self
    }

    /// Add multiple middlewares for a protocol
    pub fn middlewares<P: Protocol + 'static>(
        mut self,
        mws: Vec<Arc<dyn AsyncMiddleware<P::Context>>>,
    ) -> Self {
        for mw in mws {
            self = self.middleware::<P>(mw);
        }
        self
    }

    /// Build the SqlClient
    pub fn build(self) -> Arc<Client> {
        Arc::new(Client {
            name: self.name.unwrap_or_else(|| "sql_client".to_string()),
            connection_string: self.connection_string,
            db_type: self.db_type,
            config: self.config,
            statics: self.statics,
            middlewares: self.middlewares,
        })
    }
}

pub type SClient = Lazy<Arc<Client>>;

impl Client {
    /// Create a PostgreSQL client builder
    pub fn postgres(url: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(url, DatabaseType::Postgres)
    }

    /// Create a MySQL client builder
    pub fn mysql(url: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(url, DatabaseType::MySQL)
    }

    /// Create a SQLite client builder
    pub fn sqlite(url: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(url, DatabaseType::SQLite)
    }

    /// Get config reference
    pub fn config(&self) -> &Params {
        &self.config
    }

    /// Get statics reference
    pub fn statics(&self) -> &Locals {
        &self.statics
    }

    /// Get a config value
    pub fn get_config<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.config.get::<T>().cloned()
    }

    /// Get a static value
    pub fn get_static<T: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<T> {
        self.statics.get::<T>(key).cloned()
    }

    /// Get a config value or default
    pub fn get_config_or_default<T: Clone + Default + Send + Sync + 'static>(&self) -> T {
        self.config.get::<T>().cloned().unwrap_or_default()
    }

    /// Get a static value or default
    pub fn get_static_or_default<T: Clone + Default + Send + Sync + 'static>(&self, key: &str) -> T {
        self.statics.get::<T>(key).cloned().unwrap_or_default()
    }

    /// Get client middlewares for a protocol
    pub fn get_client_middlewares<P: Protocol + 'static>(
        &self,
    ) -> Vec<Arc<dyn AsyncMiddleware<P::Context>>> {
        self.middlewares
            .get(&TypeId::of::<P>())
            .and_then(|mws| mws.downcast_ref::<Vec<Arc<dyn AsyncMiddleware<P::Context>>>>())
            .cloned()
            .unwrap_or_default()
    }

    /// Register a query outpoint
    ///
    /// This creates an outpoint registration for SQL queries.
    /// The query name should be a path-like string (e.g., "/users/<int:id>")
    pub fn query<P: Protocol, A: AsRef<str>>(
        self: &Arc<Self>,
        query_name: A,
    ) -> OutpointRegistration<P::Context> {
        match parse(query_name.as_ref()) {
            Ok((patterns, names)) => {
                OutpointRegistration::new(self.name.clone(), patterns, names)
            }
            Err(e) => {
                crate::debug_error!("Error parsing query outpoint URL: {}", e);
                OutpointRegistration::new(self.name.clone(), Vec::new(), Vec::new())
            }
        }
    }

    /// Get the connection pool for this client
    ///
    /// Returns None if pooling is disabled or no pool exists yet.
    #[cfg(feature = "sql")]
    pub async fn pool(&self) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        use crate::sql::pool::{ConnectionPool, ConnectionKey};

        let key = ConnectionKey {
            database_url: self.connection_string.clone(),
            db_type: self.db_type.clone(),
        };
        ConnectionPool::global().get(&key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_client_builder() {
        let client = Client::postgres("postgres://localhost/test")
            .name("test_db")
            .build();

        assert_eq!(client.name, "test_db");
        assert_eq!(client.db_type, DatabaseType::Postgres);
    }

    #[test]
    fn test_mysql_client_builder() {
        let client = Client::mysql("mysql://localhost/test")
            .name("mysql_db")
            .build();

        assert_eq!(client.name, "mysql_db");
        assert_eq!(client.db_type, DatabaseType::MySQL);
    }

    #[test]
    fn test_sqlite_client_builder() {
        let client = Client::sqlite("sqlite://test.db")
            .name("sqlite_db")
            .build();

        assert_eq!(client.name, "sqlite_db");
        assert_eq!(client.db_type, DatabaseType::SQLite);
    }

    #[test]
    fn test_client_config() {
        #[derive(Clone, Debug, PartialEq)]
        struct TestConfig {
            value: String,
        }

        let client = Client::postgres("postgres://localhost/test")
            .set_config(TestConfig { value: "test".to_string() })
            .build();

        let config = client.get_config::<TestConfig>();
        assert!(config.is_some());
        assert_eq!(config.unwrap().value, "test");
    }

    #[test]
    fn test_client_statics() {
        let client = Client::postgres("postgres://localhost/test")
            .set_static("api_key", "secret123".to_string())
            .build();

        let api_key = client.get_static::<String>("api_key");
        assert_eq!(api_key, Some("secret123".to_string()));
    }
}
