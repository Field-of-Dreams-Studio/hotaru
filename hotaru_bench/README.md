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

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+ (MSRV varies by framework)
- Cargo

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
- **hotaru**: Lightweight framework (optional)
- **actix-web** & **actix-rt**: Actix framework (optional)
- **axum** & **tower**: Axum framework (optional)
- **rocket**: Rocket framework (optional)

## 🎯 Compliance

This implementation follows the TechEmpower Framework Benchmarks specifications:
- ✅ All responses include required `Server` and `Date` headers
- ✅ JSON endpoint returns proper content type and structure
- ✅ Plaintext endpoint returns proper content type
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
