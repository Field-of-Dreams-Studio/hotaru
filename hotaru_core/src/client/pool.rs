//! Connection pool implementation for HTTP client.
//!
//! This module provides a per-host connection pool to enable connection reuse
//! and reduce TCP/TLS handshake overhead. The pool is managed as a global singleton
//! and supports configurable limits for idle connections, connection lifetime, and
//! idle timeout.

use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};
use crate::connection::TcpConnectionStream;

/// Configuration for the connection pool.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum idle connections per host (default: 32)
    pub max_idle_per_host: usize,
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
            max_idle_per_host: 32,
            max_lifetime: Duration::from_secs(300),      // 5 minutes
            idle_timeout: Duration::from_secs(90),       // 90 seconds
            connection_timeout: Duration::from_secs(30), // 30 seconds
            enable_pooling: true,
        }
    }
}

/// Key for identifying connections in the pool.
///
/// Connections are pooled per unique combination of host, port, and TLS usage.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConnectionKey {
    /// Target host (domain or IP address)
    pub host: String,
    /// Target port
    pub port: u16,
    /// Whether TLS is used
    pub use_tls: bool,
}

impl ConnectionKey {
    /// Creates a new connection key.
    pub fn new(host: String, port: u16, use_tls: bool) -> Self {
        Self { host, port, use_tls }
    }
}

impl Hash for ConnectionKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.host.hash(state);
        self.port.hash(state);
        self.use_tls.hash(state);
    }
}

/// Wrapper for pooled connections with metadata.
struct PooledConnection {
    /// The underlying TCP connection stream
    stream: TcpConnectionStream,
    /// When this connection was created
    created_at: Instant,
    /// When this connection was last used
    last_used: Instant,
    /// Number of requests made on this connection
    request_count: u64,
}

impl PooledConnection {
    /// Creates a new pooled connection.
    fn new(stream: TcpConnectionStream) -> Self {
        let now = Instant::now();
        Self {
            stream,
            created_at: now,
            last_used: now,
            request_count: 0,
        }
    }

    /// Checks if the connection is still healthy based on config.
    fn is_healthy(&self, config: &PoolConfig) -> bool {
        let now = Instant::now();

        // Check if connection has exceeded max lifetime
        if now.duration_since(self.created_at) > config.max_lifetime {
            return false;
        }

        // Check if connection has been idle too long
        if now.duration_since(self.last_used) > config.idle_timeout {
            return false;
        }

        true
    }

    /// Updates the last used timestamp.
    fn touch(&mut self) {
        self.last_used = Instant::now();
    }

    /// Increments the request count.
    fn increment_requests(&mut self) {
        self.request_count += 1;
    }
}

/// Per-host connection pool with FIFO queue.
struct PerHostPool {
    /// Queue of idle connections (FIFO)
    connections: VecDeque<PooledConnection>,
    /// Maximum idle connections for this host
    max_idle: usize,
}

impl PerHostPool {
    /// Creates a new per-host pool.
    fn new(max_idle: usize) -> Self {
        Self {
            connections: VecDeque::with_capacity(max_idle),
            max_idle,
        }
    }

    /// Attempts to get a healthy connection from the pool.
    fn get(&mut self, config: &PoolConfig) -> Option<TcpConnectionStream> {
        while let Some(mut conn) = self.connections.pop_front() {
            if conn.is_healthy(config) {
                conn.touch();
                conn.increment_requests();
                return Some(conn.stream);
            }
            // Connection is stale, drop it and try next
        }
        None
    }

    /// Returns a connection to the pool.
    fn put(&mut self, stream: TcpConnectionStream) {
        // Respect max_idle limit
        if self.connections.len() >= self.max_idle {
            // Pool is full, drop the oldest connection
            self.connections.pop_front();
        }

        self.connections.push_back(PooledConnection::new(stream));
    }

    /// Removes stale connections from the pool.
    fn cleanup(&mut self, config: &PoolConfig) -> usize {
        let original_len = self.connections.len();
        self.connections.retain(|conn| conn.is_healthy(config));
        original_len - self.connections.len()
    }

    /// Returns the number of idle connections.
    fn len(&self) -> usize {
        self.connections.len()
    }

    /// Returns true if the pool is empty.
    fn is_empty(&self) -> bool {
        self.connections.is_empty()
    }
}

/// Statistics for the connection pool.
#[derive(Debug, Default, Clone)]
pub struct PoolStats {
    /// Number of successful pool hits
    pub hits: u64,
    /// Number of pool misses
    pub misses: u64,
    /// Number of connections evicted due to age/idle
    pub evictions: u64,
    /// Current number of pooled connections
    pub pooled_connections: usize,
}

/// Global connection pool singleton.
pub struct ConnectionPool {
    /// Per-host connection pools
    pools: RwLock<HashMap<ConnectionKey, PerHostPool>>,
    /// Pool configuration
    config: PoolConfig,
    /// Pool statistics
    stats: RwLock<PoolStats>,
}

impl ConnectionPool {
    /// Returns the global connection pool instance.
    pub fn global() -> &'static ConnectionPool {
        static POOL: OnceLock<ConnectionPool> = OnceLock::new();
        POOL.get_or_init(|| ConnectionPool::new(PoolConfig::default()))
    }

    /// Creates a new connection pool with the given configuration.
    fn new(config: PoolConfig) -> Self {
        Self {
            pools: RwLock::new(HashMap::new()),
            config,
            stats: RwLock::new(PoolStats::default()),
        }
    }

    /// Attempts to get a connection from the pool.
    ///
    /// Returns `Some(stream)` if a healthy connection is available,
    /// or `None` if the pool is empty or all connections are stale.
    pub async fn get(&self, key: &ConnectionKey) -> Option<TcpConnectionStream> {
        if !self.config.enable_pooling {
            return None;
        }

        let mut pools = self.pools.write().unwrap();
        let stream = pools
            .get_mut(key)
            .and_then(|pool| pool.get(&self.config));

        if stream.is_some() {
            // Pool hit
            let mut stats = self.stats.write().unwrap();
            stats.hits += 1;
        } else {
            // Pool miss
            let mut stats = self.stats.write().unwrap();
            stats.misses += 1;
        }

        stream
    }

    /// Returns a connection to the pool.
    ///
    /// If the pool is full, the oldest connection will be evicted.
    pub async fn put(&self, key: ConnectionKey, stream: TcpConnectionStream) {
        if !self.config.enable_pooling {
            return;
        }

        let mut pools = self.pools.write().unwrap();
        let pool = pools
            .entry(key)
            .or_insert_with(|| PerHostPool::new(self.config.max_idle_per_host));

        pool.put(stream);
    }

    /// Removes stale connections from all pools.
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

    /// Returns current pool statistics.
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

    /// Clears all connections from the pool.
    pub async fn clear(&self) {
        let mut pools = self.pools.write().unwrap();
        pools.clear();

        let mut stats = self.stats.write().unwrap();
        stats.pooled_connections = 0;
    }
}

/// Starts a background cleanup task that periodically removes stale connections.
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
    use tokio::net::TcpListener;

    #[test]
    fn test_connection_key_equality() {
        let key1 = ConnectionKey::new("example.com".to_string(), 443, true);
        let key2 = ConnectionKey::new("example.com".to_string(), 443, true);
        let key3 = ConnectionKey::new("example.com".to_string(), 80, false);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_connection_key_hash() {
        use std::collections::HashSet;

        let key1 = ConnectionKey::new("example.com".to_string(), 443, true);
        let key2 = ConnectionKey::new("example.com".to_string(), 443, true);

        let mut set = HashSet::new();
        set.insert(key1.clone());

        assert!(set.contains(&key2));
    }

    #[tokio::test]
    async fn test_pool_get_empty() {
        let pool = ConnectionPool::new(PoolConfig::default());
        let key = ConnectionKey::new("example.com".to_string(), 443, true);

        let result = pool.get(&key).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_pool_put_get() {
        let pool = ConnectionPool::new(PoolConfig::default());
        let key = ConnectionKey::new("localhost".to_string(), 8080, false);

        // Create a test connection
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = tokio::net::TcpStream::connect(addr).await.unwrap();
        let stream = TcpConnectionStream::new_tcp(client);

        // Put connection in pool
        pool.put(key.clone(), stream).await;

        // Get connection from pool
        let retrieved = pool.get(&key).await;
        assert!(retrieved.is_some());

        // Pool should now be empty
        let empty = pool.get(&key).await;
        assert!(empty.is_none());
    }

    #[tokio::test]
    async fn test_pool_max_idle() {
        let mut config = PoolConfig::default();
        config.max_idle_per_host = 2;
        let pool = ConnectionPool::new(config);
        let key = ConnectionKey::new("localhost".to_string(), 8080, false);

        // Create test connections
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Add 3 connections (max is 2)
        for _ in 0..3 {
            let client = tokio::net::TcpStream::connect(addr).await.unwrap();
            let stream = TcpConnectionStream::new_tcp(client);
            pool.put(key.clone(), stream).await;
        }

        // Should only have 2 connections
        let stats = pool.stats();
        assert_eq!(stats.pooled_connections, 2);
    }

    #[tokio::test]
    async fn test_pool_stale_eviction() {
        let mut config = PoolConfig::default();
        config.idle_timeout = Duration::from_millis(100);
        let pool = ConnectionPool::new(config);
        let key = ConnectionKey::new("localhost".to_string(), 8080, false);

        // Create a test connection
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = tokio::net::TcpStream::connect(addr).await.unwrap();
        let stream = TcpConnectionStream::new_tcp(client);

        // Put connection in pool
        pool.put(key.clone(), stream).await;

        // Wait for idle timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Connection should be stale
        let retrieved = pool.get(&key).await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_pooled_connection_is_healthy() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let client = tokio::net::TcpStream::connect(addr).await.unwrap();
        let stream = TcpConnectionStream::new_tcp(client);

        let conn = PooledConnection::new(stream);
        let config = PoolConfig::default();

        assert!(conn.is_healthy(&config));
    }

    #[tokio::test]
    async fn test_pool_cleanup() {
        let mut config = PoolConfig::default();
        config.idle_timeout = Duration::from_millis(100);
        let pool = ConnectionPool::new(config);
        let key = ConnectionKey::new("localhost".to_string(), 8080, false);

        // Create test connections
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let client = tokio::net::TcpStream::connect(addr).await.unwrap();
        let stream = TcpConnectionStream::new_tcp(client);
        pool.put(key.clone(), stream).await;

        // Initial state
        assert_eq!(pool.stats().pooled_connections, 1);

        // Wait for idle timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Run cleanup
        pool.cleanup().await;

        // Connection should be evicted
        assert_eq!(pool.stats().pooled_connections, 0);
        assert!(pool.stats().evictions > 0);
    }

    #[tokio::test]
    async fn test_pool_stats() {
        let pool = ConnectionPool::new(PoolConfig::default());
        let key = ConnectionKey::new("localhost".to_string(), 8080, false);

        // Initial stats
        let stats = pool.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        // Pool miss
        let _ = pool.get(&key).await;
        let stats = pool.stats();
        assert_eq!(stats.misses, 1);

        // Add connection
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let client = tokio::net::TcpStream::connect(addr).await.unwrap();
        let stream = TcpConnectionStream::new_tcp(client);
        pool.put(key.clone(), stream).await;

        // Pool hit
        let _ = pool.get(&key).await;
        let stats = pool.stats();
        assert_eq!(stats.hits, 1);
    }
}
