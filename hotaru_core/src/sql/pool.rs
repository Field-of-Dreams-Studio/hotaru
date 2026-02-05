//! SQL connection pool implementation
//!
//! This module mirrors the HTTP client connection pool pattern at
//! `/Users/jerrysu/hotaru/hotaru_core/src/client/pool.rs`
//!
//! SQL connection pooling is handled differently than TCP connections:
//! - SQLx provides its own connection pool (sqlx::Pool)
//! - We cache these pools per database URL + type
//! - Pools manage their own connection lifecycle

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};
use std::any::Any;
use std::sync::Arc;

use crate::sql::DatabaseType;

/// Configuration for SQL connection pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum idle connections per database (default: 32)
    pub max_idle_per_db: usize,
    /// Maximum connection age before eviction (default: 5 minutes)
    pub max_lifetime: Duration,
    /// Idle connection timeout (default: 90 seconds)
    pub idle_timeout: Duration,
    /// Connection creation timeout (default: 30 seconds)
    pub connection_timeout: Duration,
    /// Feature flag to enable/disable pooling (default: true)
    pub enable_pooling: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_idle_per_db: 32,
            max_lifetime: Duration::from_secs(300),      // 5 minutes
            idle_timeout: Duration::from_secs(90),       // 90 seconds
            connection_timeout: Duration::from_secs(30), // 30 seconds
            enable_pooling: true,
        }
    }
}

/// Key for identifying database connections in the pool
///
/// Connections are pooled per unique combination of database URL and type.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConnectionKey {
    /// Database connection URL
    pub database_url: String,
    /// Database type (Postgres, MySQL, SQLite)
    pub db_type: DatabaseType,
}

impl ConnectionKey {
    /// Creates a new connection key
    pub fn new(database_url: String, db_type: DatabaseType) -> Self {
        Self { database_url, db_type }
    }
}

impl Hash for ConnectionKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.database_url.hash(state);
        self.db_type.hash(state);
    }
}

/// Wrapper for pooled database connections with metadata
struct PooledDbConnection {
    /// The underlying database pool (type-erased)
    pool: Arc<dyn Any + Send + Sync>,
    /// When this pool was created
    created_at: Instant,
    /// When this pool was last used
    last_used: Instant,
    /// Number of queries executed through this pool
    query_count: u64,
}

impl PooledDbConnection {
    /// Creates a new pooled database connection
    fn new(pool: Arc<dyn Any + Send + Sync>) -> Self {
        let now = Instant::now();
        Self {
            pool,
            created_at: now,
            last_used: now,
            query_count: 0,
        }
    }

    /// Checks if the pool is still healthy based on config
    fn is_healthy(&self, config: &PoolConfig) -> bool {
        let now = Instant::now();

        // Check if pool has exceeded max lifetime
        if now.duration_since(self.created_at) > config.max_lifetime {
            return false;
        }

        // Check if pool has been idle too long
        if now.duration_since(self.last_used) > config.idle_timeout {
            return false;
        }

        true
    }

    /// Updates the last used timestamp
    fn touch(&mut self) {
        self.last_used = Instant::now();
    }

    /// Increments the query count
    fn increment_queries(&mut self) {
        self.query_count += 1;
    }
}

/// Per-database connection pool with FIFO queue
struct PerDbPool {
    /// Queue of idle pools (FIFO)
    connections: VecDeque<PooledDbConnection>,
    /// Maximum idle pools for this database
    max_idle: usize,
}

impl PerDbPool {
    /// Creates a new per-database pool
    fn new(max_idle: usize) -> Self {
        Self {
            connections: VecDeque::with_capacity(max_idle),
            max_idle,
        }
    }

    /// Attempts to get a healthy pool from storage
    fn get(&mut self, config: &PoolConfig) -> Option<Arc<dyn Any + Send + Sync>> {
        let mut index = 0;
        while index < self.connections.len() {
            if !self.connections[index].is_healthy(config) {
                self.connections.remove(index);
                continue;
            }

            let conn = &mut self.connections[index];
            conn.touch();
            conn.increment_queries();
            return Some(conn.pool.clone());
        }
        None
    }

    /// Stores a pool
    fn put(&mut self, pool: Arc<dyn Any + Send + Sync>) {
        if self.connections.len() >= self.max_idle {
            // Pool is full, drop the oldest
            self.connections.pop_front();
        }
        self.connections.push_back(PooledDbConnection::new(pool));
    }

    /// Removes stale pools
    fn cleanup(&mut self, config: &PoolConfig) -> usize {
        let original_len = self.connections.len();
        self.connections.retain(|conn| conn.is_healthy(config));
        original_len - self.connections.len()
    }

    /// Returns whether the pool is empty
    fn is_empty(&self) -> bool {
        self.connections.is_empty()
    }

    /// Returns number of pooled connections
    fn len(&self) -> usize {
        self.connections.len()
    }
}

/// Statistics for the connection pool
#[derive(Debug, Default, Clone)]
pub struct PoolStats {
    /// Number of successful pool hits
    pub hits: u64,
    /// Number of pool misses
    pub misses: u64,
    /// Number of pools evicted due to age/idle
    pub evictions: u64,
    /// Current number of pooled connections
    pub pooled_connections: usize,
}

/// Global SQL connection pool singleton
pub struct ConnectionPool {
    /// Per-database connection pools
    pools: RwLock<HashMap<ConnectionKey, PerDbPool>>,
    /// Pool configuration
    config: PoolConfig,
    /// Pool statistics
    stats: RwLock<PoolStats>,
}

impl ConnectionPool {
    /// Returns the global connection pool instance
    pub fn global() -> &'static ConnectionPool {
        static POOL: OnceLock<ConnectionPool> = OnceLock::new();
        POOL.get_or_init(|| ConnectionPool::new(PoolConfig::default()))
    }

    /// Creates a new connection pool with the given configuration
    fn new(config: PoolConfig) -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            config,
            stats: RwLock::new(PoolStats::default()),
        }
    }

    /// Attempts to get a pool from the global cache
    ///
    /// Returns `Some(pool)` if a healthy pool is available,
    /// or `None` if no pool exists or it's stale.
    pub async fn get(&self, key: &ConnectionKey) -> Option<Arc<dyn Any + Send + Sync>> {
        if !self.config.enable_pooling {
            return None;
        }

        let mut pools = self.pools.write().unwrap();
        let pool = pools
            .get_mut(key)
            .and_then(|pool| pool.get(&self.config));

        if pool.is_some() {
            // Pool hit
            let mut stats = self.stats.write().unwrap();
            stats.hits += 1;
        } else {
            // Pool miss
            let mut stats = self.stats.write().unwrap();
            stats.misses += 1;
        }

        pool
    }

    /// Stores a pool in the global cache
    pub async fn put(&self, key: ConnectionKey, pool: Arc<dyn Any + Send + Sync>) {
        if !self.config.enable_pooling {
            return;
        }

        let mut pools = self.pools.write().unwrap();
        let db_pool = pools
            .entry(key)
            .or_insert_with(|| PerDbPool::new(self.config.max_idle_per_db));

        db_pool.put(pool);
    }

    /// Removes stale pools from all databases
    ///
    /// This should be called periodically by a background task.
    pub async fn cleanup(&self) {
        let mut pools = self.pools.write().unwrap();
        let mut total_evictions = 0;

        // Clean up each pool and remove empty pools
        pools.retain(|_, pool| {
            total_evictions += pool.cleanup(&self.config);
            !pool.is_empty()
        });

        // Update statistics
        let mut stats = self.stats.write().unwrap();
        stats.evictions += total_evictions as u64;
        stats.pooled_connections = pools.values().map(|p| p.len()).sum();
    }

    /// Returns current pool statistics
    pub fn stats(&self) -> PoolStats {
        let stats = self.stats.read().unwrap();
        let pools = self.pools.read().unwrap();

        PoolStats {
            hits: stats.hits,
            misses: stats.misses,
            evictions: stats.evictions,
            pooled_connections: pools.values().map(|p| p.len()).sum(),
        }
    }

    /// Clears all pools from the cache
    pub async fn clear(&self) {
        let mut pools = self.pools.write().unwrap();
        pools.clear();

        let mut stats = self.stats.write().unwrap();
        stats.pooled_connections = 0;
    }
}

/// Starts a background cleanup task that periodically removes stale pools
///
/// This should be called once during application initialization.
pub fn start_cleanup_task() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            ConnectionPool::global().cleanup().await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_key_equality() {
        let key1 = ConnectionKey::new(
            "postgres://localhost/db".to_string(),
            DatabaseType::Postgres,
        );
        let key2 = ConnectionKey::new(
            "postgres://localhost/db".to_string(),
            DatabaseType::Postgres,
        );
        let key3 = ConnectionKey::new(
            "mysql://localhost/db".to_string(),
            DatabaseType::MySQL,
        );

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_connection_key_hash() {
        use std::collections::HashSet;

        let key1 = ConnectionKey::new(
            "postgres://localhost/db".to_string(),
            DatabaseType::Postgres,
        );
        let key2 = ConnectionKey::new(
            "postgres://localhost/db".to_string(),
            DatabaseType::Postgres,
        );

        let mut set = HashSet::new();
        set.insert(key1.clone());

        assert!(set.contains(&key2));
    }

    #[tokio::test]
    async fn test_pool_get_empty() {
        let pool = ConnectionPool::new(PoolConfig::default());
        let key = ConnectionKey::new(
            "postgres://localhost/test".to_string(),
            DatabaseType::Postgres,
        );

        let result = pool.get(&key).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_pool_put_get() {
        let pool = ConnectionPool::new(PoolConfig::default());
        let key = ConnectionKey::new(
            "postgres://localhost/test".to_string(),
            DatabaseType::Postgres,
        );

        let db_pool: Arc<dyn Any + Send + Sync> = Arc::new("pool1".to_string());
        pool.put(key.clone(), db_pool).await;

        let retrieved = pool.get(&key).await;
        assert!(retrieved.is_some());

        let second = pool.get(&key).await;
        assert!(second.is_some());
    }

    #[tokio::test]
    async fn test_pool_max_idle() {
        let mut config = PoolConfig::default();
        config.max_idle_per_db = 2;
        let pool = ConnectionPool::new(config);
        let key = ConnectionKey::new(
            "postgres://localhost/test".to_string(),
            DatabaseType::Postgres,
        );

        for i in 0..3 {
            let db_pool: Arc<dyn Any + Send + Sync> = Arc::new(format!("pool{}", i));
            pool.put(key.clone(), db_pool).await;
        }

        let stats = pool.stats();
        assert_eq!(stats.pooled_connections, 2);
    }

    #[tokio::test]
    async fn test_pool_stale_eviction() {
        let mut config = PoolConfig::default();
        config.idle_timeout = Duration::from_millis(100);
        let pool = ConnectionPool::new(config);
        let key = ConnectionKey::new(
            "postgres://localhost/test".to_string(),
            DatabaseType::Postgres,
        );

        let db_pool: Arc<dyn Any + Send + Sync> = Arc::new("pool1".to_string());
        pool.put(key.clone(), db_pool).await;

        tokio::time::sleep(Duration::from_millis(150)).await;

        let retrieved = pool.get(&key).await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_pool_cleanup() {
        let mut config = PoolConfig::default();
        config.idle_timeout = Duration::from_millis(100);
        let pool = ConnectionPool::new(config);
        let key = ConnectionKey::new(
            "postgres://localhost/test".to_string(),
            DatabaseType::Postgres,
        );

        let db_pool: Arc<dyn Any + Send + Sync> = Arc::new("pool1".to_string());
        pool.put(key.clone(), db_pool).await;

        assert_eq!(pool.stats().pooled_connections, 1);

        tokio::time::sleep(Duration::from_millis(150)).await;
        pool.cleanup().await;

        assert_eq!(pool.stats().pooled_connections, 0);
        assert!(pool.stats().evictions > 0);
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let pool = ConnectionPool::new(PoolConfig::default());
        let key = ConnectionKey::new(
            "sqlite://test.db".to_string(),
            DatabaseType::SQLite,
        );

        // Initial stats
        let stats = pool.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        // Pool miss
        let _ = pool.get(&key).await;
        let stats = pool.stats();
        assert_eq!(stats.misses, 1);

        // Add pool
        let db_pool: Arc<dyn Any + Send + Sync> = Arc::new("pool1".to_string());
        pool.put(key.clone(), db_pool).await;

        // Pool hit
        let _ = pool.get(&key).await;
        let stats = pool.stats();
        assert_eq!(stats.hits, 1);
    }
}
