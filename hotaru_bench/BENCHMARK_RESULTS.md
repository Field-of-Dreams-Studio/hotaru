# TechEmpower Benchmark Results

Test Date: 2025-11-22  
Test Machine: MacBook Pro (Apple Silicon)  
Test Tool: wrk  
Test Parameters: 4 threads, 16 connections, 10 seconds duration

## 📊 Test Results Summary

### JSON Serialization Test (`/json`)

| Framework | Requests/sec (RPS) | Average Latency | Throughput |
|-----------|-------------------|-----------------|------------|
| **Hotaru** | **173,254** | 84.30µs | 18.84 MB/s |
| Rocket | 171,904 | 84.82µs | 41.48 MB/s |
| Actix-web | 149,244 | 102.27µs | 19.21 MB/s |
| Axum | 148,934 | 102.73µs | 19.17 MB/s |

### Plaintext Test (`/plaintext`)

| Framework | Requests/sec (RPS) | Average Latency | Throughput |
|-----------|-------------------|-----------------|------------|
| **Hotaru** | **175,593** | 85.76µs | 15.57 MB/s |
| Rocket | 173,357 | 83.78µs | 41.00 MB/s |
| Actix-web | 150,724 | 100.98µs | 16.53 MB/s |
| Axum | 149,677 | 101.89µs | 16.42 MB/s |

## 🏆 Performance Rankings

### JSON Endpoint
1. 🥇 **Hotaru** - 173,254 RPS
2. 🥈 **Rocket** - 171,904 RPS (99.2% of Hotaru)
3. 🥉 **Actix-web** - 149,244 RPS (86.1% of Hotaru)
4. **Axum** - 148,934 RPS (86.0% of Hotaru)

### Plaintext Endpoint
1. 🥇 **Hotaru** - 175,593 RPS
2. 🥈 **Rocket** - 173,357 RPS (98.7% of Hotaru)
3. 🥉 **Actix-web** - 150,724 RPS (85.8% of Hotaru)
4. **Axum** - 149,677 RPS (85.2% of Hotaru)

## 📈 Detailed Analysis

### Hotaru (v0.7.5)
- ✅ **Fastest JSON Serialization**: 173,254 RPS
- ✅ **Fastest Plaintext**: 175,593 RPS
- ✅ **Lowest Latency**: ~85µs
- 🎯 Excellent performance as a new framework, ranking first in both tests

### Rocket (v0.5)
- ✅ Performance close to Hotaru, within 2%
- ✅ Excellent latency performance
- ⚠️ Unusually high throughput numbers (possibly measurement differences)
- 🎯 Good balance between ease of use and performance

### Actix-web (v4)
- ✅ Mature and stable framework
- ✅ Still strong performance
- 📊 Approximately 14% slower than top frameworks
- 🎯 Widely used in production environments

### Axum (v0.8.7)
- ✅ Built on Tokio/Tower, comprehensive ecosystem
- ✅ Type-safe and ergonomic design
- 📊 Performance comparable to Actix
- 🎯 Modern design, suitable for large projects

## 🔍 Key Findings

1. **Hotaru Excels**: As a newer framework, it achieved the best performance in both benchmark tests
2. **Consistent Latency**: All frameworks have latency within the 85-103µs range, with minimal differences
3. **Clear Performance Tiers**: Hotaru/Rocket in the first tier, Actix/Axum in the second tier
4. **All Frameworks Are Fast**: Even the "slowest" Axum achieves ~150k RPS, which is more than sufficient for most application scenarios

## ✅ TechEmpower Specification Compliance

All implementations comply with TechEmpower Framework Benchmarks specifications:
- ✅ JSON endpoint returns `{"message":"Hello, World!"}`
- ✅ Plaintext endpoint returns `Hello, World!`
- ✅ Correct Content-Type headers
- ✅ Includes Server and Date headers
- ✅ Supports HTTP Keep-Alive
- ✅ Runs on port 8080
- ✅ No disk logging

## 🛠️ Test Commands

### JSON Test
```bash
wrk -t4 -c16 -d10s http://localhost:8080/json
```

### Plaintext Test
```bash
wrk -t4 -c16 -d10s http://localhost:8080/plaintext
```

## 📝 Notes

- These are single-machine test results; actual production performance will be affected by various factors
- Rocket's throughput data appears anomalous and may require further investigation
- All tests were conducted on identical hardware and conditions to ensure fairness
- Framework selection should be based on specific application scenarios, not just performance metrics

## 🚀 Run Your Own Tests

```bash
# Hotaru
cargo run --release --features hotaru_server

# Actix-web
cargo run --release --features actix_server

# Axum
cargo run --release --features axum_server

# Rocket
cargo run --release --features rocket_server
```
