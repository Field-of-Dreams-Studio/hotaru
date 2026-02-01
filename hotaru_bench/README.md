# TechEmpower Benchmark Suite for Rust Web Frameworks

This project implements the TechEmpower Framework Benchmarks for four popular Rust web frameworks:
- **Hotaru** - A lightweight, intuitive web framework
- **Actix-web** - A powerful, pragmatic, and extremely fast web framework
- **Axum** - An ergonomic and modular web framework built with Tokio and Tower
- **Rocket** - A web framework for Rust that makes it simple to write fast, secure web applications

## 📋 Implemented Tests

According to the [TechEmpower Framework Benchmarks specification](https://github.com/TechEmpower/FrameworkBenchmarks/wiki/Project-Information-Framework-Tests-Overview), we implement:

### 1. JSON Serialization Test (`/json`)
- **Endpoint**: `GET /json`
- **Response**: `{"message":"Hello, World!"}`
- **Content-Type**: `application/json`
- Tests framework fundamentals including routing, JSON serialization, and response handling

### 2. Plaintext Test (`/plaintext`)
- **Endpoint**: `GET /plaintext`
- **Response**: `Hello, World!`
- **Content-Type**: `text/plain`
- Tests raw request routing and response generation capabilities

### 3. Single Database Query Test (`/db`)
- **Endpoint**: `GET /db`
- **Response**: `{"id": 123, "randomNumber": 456}`
- **Content-Type**: `application/json`
- Fetches one random World row (ID 1-10000)

### 4. Multiple Queries Test (`/queries`)
- **Endpoint**: `GET /queries?queries=N`
- **Response**: Array of World objects
- **Constraints**: `queries` clamped to 1-500

### 5. Updates Test (`/updates`)
- **Endpoint**: `GET /updates?queries=N`
- **Response**: Array of updated World objects
- **Constraints**: `queries` clamped to 1-500

### 6. Cached Queries Test (`/cached-worlds`)
- **Endpoint**: `GET /cached-worlds?count=N`
- **Response**: Array of World objects
- **Constraints**: `count` clamped to 1-500
- **Cache**: Pre-warmed with all 10,000 CachedWorld rows

### 7. Fortunes Test (`/fortunes`)
- **Endpoint**: `GET /fortunes`
- **Response**: HTML table of fortunes
- **Content-Type**: `text/html; charset=utf-8`
- Adds an extra fortune at request time and sorts by message

## 🛠️ Tech Stack

### Core Technologies
- **Language**: Rust 2024 Edition
- **Async Runtime**: Tokio
- **Serialization**: Serde + serde_json

### Frameworks
- **Hotaru** 0.7.x - Lightweight framework with Akari templates
- **Actix-web** 4.x - Actor-based framework
- **Axum** 0.8.x - Tower-based framework
- **Rocket** 0.5.x - Batteries-included framework

### Database Layer
- **Database**: PostgreSQL 15+
- **Driver**: tokio-postgres 0.7
- **Connection Pool**: deadpool-postgres 0.14
- **Caching**: moka 0.12

### Templating
- **Hotaru**: Akari 0.2
- **Actix/Axum**: Askama 0.12
- **Rocket**: Tera (via rocket_dyn_templates)

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+ (MSRV varies by framework)
- Cargo
- PostgreSQL 15+ (for database-backed tests)

### Database Setup

The database-backed TechEmpower tests expect a `hello_world` database and the default credentials used by the benchmark suite.

```bash
# Create database and user (example for local Postgres installs)
sudo -u postgres createdb hello_world
sudo -u postgres createuser benchmarkdbuser
sudo -u postgres psql -c "ALTER USER benchmarkdbuser WITH PASSWORD 'benchmarkdbpass';"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE hello_world TO benchmarkdbuser;"

# Create tables and seed data
cd hotaru_bench/db
./setup.sh
```

Environment variables (see `.env.example`):
```
DATABASE_URL=postgres://benchmarkdbuser:benchmarkdbpass@localhost/hello_world
DB_POOL_SIZE=56
```

### Running Benchmarks

Each framework can be run independently using feature flags:

#### Hotaru
```bash
cargo run --release --features hotaru_server
```

#### Actix-web
```bash
cargo run --release --features actix_server
```

#### Axum
```bash
cargo run --release --features axum_server
```

#### Rocket
```bash
cargo run --release --features rocket_server
```

All servers run on `http://0.0.0.0:8080`

For database-backed tests, set `DATABASE_URL` (and optionally `DB_POOL_SIZE`) before running.

## 🧪 Testing Endpoints

Once a server is running, you can test the endpoints:

### Test JSON endpoint
```bash
curl http://localhost:8080/json
# Expected: {"message":"Hello, World!"}
```

### Test Plaintext endpoint
```bash
curl http://localhost:8080/plaintext
# Expected: Hello, World!
```

### Test Database endpoints
```bash
curl http://localhost:8080/db
curl "http://localhost:8080/queries?queries=10"
curl "http://localhost:8080/updates?queries=5"
curl "http://localhost:8080/cached-worlds?count=100"
curl http://localhost:8080/fortunes
```

### Using wrk for benchmarking
```bash
# Install wrk
# macOS: brew install wrk
# Linux: sudo apt-get install wrk

# Benchmark JSON endpoint
wrk -t4 -c100 -d30s http://localhost:8080/json

# Benchmark Plaintext endpoint
wrk -t4 -c100 -d30s http://localhost:8080/plaintext
```

### Using Apache Bench (ab)
```bash
# JSON endpoint
ab -n 10000 -c 100 http://localhost:8080/json

# Plaintext endpoint
ab -n 10000 -c 100 http://localhost:8080/plaintext
```

## 📊 Benchmark Methodology

According to TechEmpower specifications:

1. **JSON Serialization Test**:
   - Must instantiate an object for each request
   - Must serialize to JSON (no caching)
   - Response must be exactly `{"message":"Hello, World!"}`
   - Must include `Server` and `Date` headers
   - Must specify `Content-Length` or `Transfer-Encoding`

2. **Plaintext Test**:
   - Response must be `Hello, World!`
   - Content-Type: `text/plain`
   - Tests request routing fundamentals
   - Designed for HTTP pipelining

3. **Database Tests**:
   - `/db`, `/queries`, `/updates` execute separate queries per World row
   - `/cached-worlds` served from in-memory cache (no DB access on request)
   - `/fortunes` renders HTML with XSS-escaped content

## 🔧 Build Options

### Development build
```bash
cargo build --features <framework>_server
```

### Release build (recommended for benchmarking)
```bash
cargo build --release --features <framework>_server
```

## 📦 Dependencies

- **serde** & **serde_json**: JSON serialization
- **tokio**: Async runtime (required by all frameworks)
- **tokio-postgres** & **deadpool-postgres**: PostgreSQL driver and connection pool
- **moka**: High-performance in-memory cache for cached-worlds
- **askama** / **tera**: HTML templating for fortunes
- **hotaru**: Lightweight framework (optional)
- **actix-web** & **actix-rt**: Actix framework (optional)
- **axum** & **tower**: Axum framework (optional)
- **rocket**: Rocket framework (optional)

## 🎯 Compliance

This implementation follows the TechEmpower Framework Benchmarks specifications:
- ✅ All responses include required `Server` and `Date` headers
- ✅ JSON endpoint returns proper content type and structure
- ✅ Plaintext endpoint returns proper content type
- ✅ Database endpoints clamp query counts to 1-500
- ✅ Fortunes endpoint HTML-escapes messages
- ✅ No disk logging enabled
- ✅ Servers listen on 4-digit port (8080)
- ✅ HTTP Keep-Alive supported
- ✅ Production-grade implementations

## 📖 Framework Documentation

- **Hotaru**: https://docs.rs/hotaru / https://github.com/Field-of-Dreams-Studio/hotaru
- **Actix-web**: https://actix.rs/
- **Axum**: https://docs.rs/axum/
- **Rocket**: https://rocket.rs/

## 📝 Notes

- All implementations use async/await with Tokio
- Feature flags allow building only one framework at a time to reduce binary size
- Default feature builds nothing - you must specify a framework feature
- All endpoints conform to TechEmpower benchmark requirements

## 🔗 References

- [TechEmpower Framework Benchmarks](https://www.techempower.com/benchmarks/)
- [Test Type Specifications](https://github.com/TechEmpower/FrameworkBenchmarks/wiki/Project-Information-Framework-Tests-Overview)
- [TechEmpower GitHub Repository](https://github.com/TechEmpower/FrameworkBenchmarks)
