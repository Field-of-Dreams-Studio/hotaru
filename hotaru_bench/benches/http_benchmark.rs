use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hotaru_bench::{traits_bench, memory_bench, load_test::*};
use std::time::Duration;
use tokio::runtime::Runtime;

fn benchmark_traits(c: &mut Criterion) {
    let mut group = c.benchmark_group("trait_dispatch");
    
    // Run the trait benchmark
    group.bench_function("zero_cost_abstraction", |b| {
        b.iter(|| {
            traits_bench::run_trait_benchmark();
        });
    });
    
    group.finish();
}

fn benchmark_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    group.bench_function("memory_comparison", |b| {
        b.iter(|| {
            memory_bench::run_memory_benchmark();
        });
    });
    
    group.finish();
}

async fn run_framework_benchmark(name: &str, port: u16) -> LoadTestResult {
    let config = LoadTestConfig {
        url: format!("http://127.0.0.1:{}/", port),
        requests: 10000,
        concurrency: 100,
        warmup_requests: 100,
    };
    
    println!("\nBenchmarking {}", name);
    run_load_test(config).await
}

fn benchmark_http_frameworks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("http_frameworks");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);
    
    // Note: These servers should be running before benchmarking
    println!("\n==================================");
    println!("HTTP Framework Benchmark");
    println!("==================================");
    println!("\n⚠️  Make sure to start servers first:");
    println!("  1. Terminal 1: cd hotaru_hello && cargo run --release");
    println!("  2. Terminal 2: cd actix_hello && cargo run --release");
    println!("\nPress Enter when servers are running...");
    
    // Uncomment to wait for user input
    // let mut input = String::new();
    // std::io::stdin().read_line(&mut input).unwrap();
    
    group.bench_function("hotaru", |b| {
        b.to_async(&rt).iter(|| async {
            let config = LoadTestConfig {
                url: "http://127.0.0.1:8001/".to_string(),
                requests: 1000,
                concurrency: 10,
                warmup_requests: 10,
            };
            run_load_test(config).await
        });
    });
    
    group.bench_function("actix_web", |b| {
        b.to_async(&rt).iter(|| async {
            let config = LoadTestConfig {
                url: "http://127.0.0.1:8002/".to_string(),
                requests: 1000,
                concurrency: 10,
                warmup_requests: 10,
            };
            run_load_test(config).await
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_traits,
    benchmark_memory,
    benchmark_http_frameworks
);
criterion_main!(benches);