use hotaru_bench::{traits_bench, memory_bench, load_test::*};
use std::process::{Command, Child};
use std::thread;
use std::time::Duration;
use tokio;

fn start_server(dir: &str, name: &str) -> std::io::Result<Child> {
    println!("Starting {} server...", name);
    Command::new("cargo")
        .args(&["run", "--release"])
        .current_dir(dir)
        .spawn()
}

async fn benchmark_framework(name: &str, port: u16) {
    println!("\n{}", "=".repeat(50));
    println!("Benchmarking: {}", name);
    println!("{:=<50}", "=");
    
    // Test plain text endpoint
    let config = LoadTestConfig {
        url: format!("http://127.0.0.1:{}/", port),
        requests: 10000,
        concurrency: 100,
        warmup_requests: 100,
    };
    
    let result = run_load_test(config).await;
    print_load_test_results(&format!("{} - Plain Text", name), &result);
    
    // Test JSON endpoint
    let config = LoadTestConfig {
        url: format!("http://127.0.0.1:{}/json", port),
        requests: 10000,
        concurrency: 100,
        warmup_requests: 100,
    };
    
    let result = run_load_test(config).await;
    print_load_test_results(&format!("{} - JSON", name), &result);
}

#[tokio::main]
async fn main() {
    println!("====================================");
    println!("Hotaru Framework Benchmark Suite");
    println!("====================================");
    
    // Run trait benchmarks
    println!("\n1. Running Trait Dispatch Benchmarks");
    traits_bench::run_trait_benchmark();
    
    // Run memory benchmarks
    println!("\n2. Running Memory Usage Benchmarks");
    memory_bench::run_memory_benchmark();
    
    // HTTP Framework benchmarks
    println!("\n3. Running HTTP Framework Benchmarks");
    println!("Starting servers...");
    
    // Build servers first
    println!("Building Hotaru server...");
    Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir("hotaru_bench/hotaru_hello")
        .status()
        .expect("Failed to build Hotaru server");
    
    println!("Building Actix-web server...");
    Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir("hotaru_bench/actix_hello")
        .status()
        .expect("Failed to build Actix-web server");
    
    println!("Building H2per server...");
    Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir("hotaru_bench/h2per_hello")
        .status()
        .expect("Failed to build H2per server");
    
    // Start servers
    let mut hotaru_server = start_server("hotaru_bench/hotaru_hello", "Hotaru")
        .expect("Failed to start Hotaru server");
    
    let mut actix_server = start_server("hotaru_bench/actix_hello", "Actix-web")
        .expect("Failed to start Actix-web server");
    
    let mut h2per_server = start_server("hotaru_bench/h2per_hello", "H2per")
        .expect("Failed to start H2per server");
    
    // Wait for servers to start
    println!("Waiting for servers to initialize...");
    thread::sleep(Duration::from_secs(5));
    
    // Check if servers are responding
    let client = reqwest::Client::new();
    
    // Check Hotaru
    match client.get("http://127.0.0.1:8001/").send().await {
        Ok(_) => println!("✓ Hotaru server is responding"),
        Err(e) => {
            eprintln!("✗ Hotaru server not responding: {}", e);
            hotaru_server.kill().ok();
            actix_server.kill().ok();
            return;
        }
    }
    
    // Check Actix-web
    match client.get("http://127.0.0.1:8002/").send().await {
        Ok(_) => println!("✓ Actix-web server is responding"),
        Err(e) => {
            eprintln!("✗ Actix-web server not responding: {}", e);
            hotaru_server.kill().ok();
            actix_server.kill().ok();
            return;
        }
    }
    
    // Check H2per
    match client.get("http://127.0.0.1:8003/").send().await {
        Ok(_) => println!("✓ H2per server is responding"),
        Err(e) => {
            eprintln!("✗ H2per server not responding: {}", e);
            hotaru_server.kill().ok();
            actix_server.kill().ok();
            h2per_server.kill().ok();
            return;
        }
    }
    
    // Run benchmarks
    benchmark_framework("Hotaru", 8001).await;
    benchmark_framework("Actix-web", 8002).await;
    benchmark_framework("H2per (Hyper-based)", 8003).await;
    
    // Compare results
    println!("\n{}", "=".repeat(50));
    println!("Summary");
    println!("{:=<50}", "=");
    println!("\nNote: These results should be taken as relative comparisons.");
    println!("Actual performance depends on many factors including:");
    println!("- Hardware specifications");
    println!("- OS and kernel configuration");
    println!("- Network stack tuning");
    println!("- Compiler optimizations");
    
    // Cleanup
    println!("\nCleaning up...");
    hotaru_server.kill().ok();
    actix_server.kill().ok();
    h2per_server.kill().ok();
    
    println!("\n✓ Benchmark suite completed!");
}