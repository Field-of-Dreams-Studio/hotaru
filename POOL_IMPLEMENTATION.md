# Connection Pool Implementation - Complete

## Summary

Successfully implemented a high-performance, per-host connection pool for the Hotaru HTTP client to enable connection reuse and reduce TCP/TLS handshake overhead.

## Implementation Details

### Files Created

1. **`hotaru_core/src/client/pool.rs`** (~500 lines)
   - Core connection pool implementation
   - Global singleton pattern using `OnceLock`
   - Per-host connection queues with FIFO semantics
   - Health checks and lifecycle management
   - Comprehensive test suite (9 tests, all passing)

2. **`hotaru_core/examples/connection_pool_demo.rs`**
   - Demonstration of connection pooling
   - Shows pool statistics and hit rates
   - Makes sequential requests to demonstrate reuse

### Files Modified

1. **`hotaru_core/src/client/mod.rs`**
   - Added pool module export
   - Exported `ConnectionPool`, `PoolConfig`, `ConnectionKey`

2. **`hotaru_core/src/http/context.rs`**
   - Modified `HttpContext::send_request()` to use connection pool
   - Added connection key creation
   - Implemented pool get/put logic
   - Added keep-alive detection from response headers

3. **`hotaru_core/src/app/application.rs`**
   - Added `start_cleanup_task()` call in `run_app_loop()`
   - Starts background task to remove stale connections

## Architecture

### Connection Pooling Strategy
- **Per-host pooling**: Separate pools for each unique `(host, port, tls)` combination
- **FIFO queue**: Oldest connections used first
- **Health checks**: Connections validated before reuse
- **Background cleanup**: Periodic task removes stale connections

### Configuration (Default Values)
```rust
PoolConfig {
    max_idle_per_host: 32,           // Max idle connections per host
    max_lifetime: 5 minutes,         // Max connection age
    idle_timeout: 90 seconds,        // Idle connection timeout
    connection_timeout: 30 seconds,  // Connection creation timeout
    enable_pooling: true,            // Feature flag
}
```

### Key Components

#### ConnectionKey
```rust
ConnectionKey {
    host: String,
    port: u16,
    use_tls: bool,
}
```
- Hash key for pool lookup
- Implements `Hash` and `Eq` for HashMap storage

#### PooledConnection
```rust
PooledConnection {
    stream: TcpConnectionStream,
    created_at: Instant,
    last_used: Instant,
    request_count: u64,
}
```
- Wraps connections with metadata
- Tracks age and usage for health checks

#### PerHostPool
```rust
PerHostPool {
    connections: VecDeque<PooledConnection>,
    max_idle: usize,
}
```
- FIFO queue of idle connections
- Enforces per-host limits

#### ConnectionPool (Singleton)
```rust
ConnectionPool {
    pools: RwLock<HashMap<ConnectionKey, PerHostPool>>,
    config: PoolConfig,
    stats: RwLock<PoolStats>,
}
```
- Global singleton accessed via `ConnectionPool::global()`
- Thread-safe via `RwLock`
- Tracks statistics (hits, misses, evictions)

## Request Flow

### With Connection Pool

1. **Request arrives** → Create `ConnectionKey` from (host, port, tls)
2. **Pool lookup** → Try `ConnectionPool::global().get(&key)`
3. **Pool hit** → Reuse existing connection
4. **Pool miss** → Create new connection via `ConnectionBuilder`
5. **Send request** → Use connection (pooled or new)
6. **Read response** → Check `Connection` header
7. **Keep-alive?** →
   - Yes → Return connection to pool via `put()`
   - No → Drop connection

### Health Validation
```rust
fn is_healthy(&self, config: &PoolConfig) -> bool {
    let now = Instant::now();

    // Check max lifetime
    if now.duration_since(self.created_at) > config.max_lifetime {
        return false;
    }

    // Check idle timeout
    if now.duration_since(self.last_used) > config.idle_timeout {
        return false;
    }

    true
}
```

## Test Results

All 9 pool tests passing:
```
test client::pool::tests::test_connection_key_equality ... ok
test client::pool::tests::test_connection_key_hash ... ok
test client::pool::tests::test_pool_get_empty ... ok
test client::pool::tests::test_pooled_connection_is_healthy ... ok
test client::pool::tests::test_pool_put_get ... ok
test client::pool::tests::test_pool_stats ... ok
test client::pool::tests::test_pool_max_idle ... ok
test client::pool::tests::test_pool_cleanup ... ok
test client::pool::tests::test_pool_stale_eviction ... ok
```

## Build Status

```bash
✓ hotaru_core builds successfully
✓ All pool tests pass
✓ Connection pool demo example compiles
✓ No new dependencies added
```

## Performance Impact

### Expected Improvements
- **Latency reduction**: 50-100ms per request (TCP + TLS handshake savings)
- **Server load**: Reduced connection establishment overhead
- **Memory**: Minimal increase (~32 connections × ~4KB = ~128KB per host)

### Trade-offs
- **Pros**:
  - Significant latency reduction for sequential requests
  - Reduced server load
  - Simple implementation (~500 LOC)
  - No new dependencies
  - Follows existing Hotaru patterns

- **Cons**:
  - Adds complexity to connection lifecycle
  - Memory overhead for idle connections (mitigated by limits)
  - Potential for stale connections (mitigated by health checks)

## Usage Example

```rust
use hotaru_core::client::pool::ConnectionPool;
use hotaru_core::http::context::HttpContext;
use hotaru_core::http::request::request_templates::get_request;
use hotaru_core::http::safety::HttpSafety;

// Make multiple requests to the same host
let request1 = get_request("/api/users");
let response1 = HttpContext::send_request(
    "https://api.example.com",
    request1,
    HttpSafety::default()
).await?;

// Second request reuses the connection!
let request2 = get_request("/api/posts");
let response2 = HttpContext::send_request(
    "https://api.example.com",
    request2,
    HttpSafety::default()
).await?;

// Check pool statistics
let stats = ConnectionPool::global().stats();
println!("Pool hits: {}", stats.hits);  // Should be >= 1
```

## Background Cleanup

The cleanup task runs every 30 seconds:
```rust
start_cleanup_task(); // Called in App::run_app_loop()

// Spawns:
tokio::spawn(async {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        ConnectionPool::global().cleanup().await;
    }
});
```

## Statistics Tracking

```rust
pub struct PoolStats {
    pub hits: u64,           // Successful pool hits
    pub misses: u64,         // Pool misses (new connections)
    pub evictions: u64,      // Connections evicted (stale)
    pub pooled_connections: usize,  // Current pooled count
}

// Access via:
let stats = ConnectionPool::global().stats();
```

## Future Enhancements (Optional)

1. **Configuration per client**: Allow per-client pool config
2. **Connection health ping**: Test connections before reuse
3. **Metrics export**: Expose pool stats via metrics API
4. **Environment variables**: Support `HOTARU_POOL_*` env vars
5. **Connection limits**: Global maximum across all hosts

## Verification Commands

```bash
# Build core library
cargo build --release --package hotaru_core

# Run pool tests
cargo test --package hotaru_core pool

# Run demo (requires internet)
cargo run --release --package hotaru_core --example connection_pool_demo
```

## References

- **Implementation point**: `hotaru_core/src/http/context.rs:802-842`
- **Pool module**: `hotaru_core/src/client/pool.rs`
- **Tests**: `hotaru_core/src/client/pool.rs:330-492`
- **Demo**: `hotaru_core/examples/connection_pool_demo.rs`

---

**Status**: ✅ Complete and tested
**Date**: 2026-02-04
**Version**: Hotaru v0.8.0
