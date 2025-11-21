// Trait-based zero-cost abstraction benchmarks
use std::hint::black_box;
use std::time::Instant;

// Test trait-based abstraction (zero-cost claim)
pub trait Protocol {
    fn process(&self, data: &[u8]) -> usize;
}

pub struct HttpProtocol;
impl Protocol for HttpProtocol {
    #[inline]
    fn process(&self, data: &[u8]) -> usize {
        data.len()
    }
}

// Dynamic dispatch version for comparison
pub fn process_dynamic(protocol: &dyn Protocol, data: &[u8]) -> usize {
    protocol.process(data)
}

// Static dispatch version (what Hotaru uses)
pub fn process_static<P: Protocol>(protocol: &P, data: &[u8]) -> usize {
    protocol.process(data)
}

pub fn run_trait_benchmark() {
    println!("\n=== Trait Dispatch Benchmark ===");
    let data = vec![0u8; 1024];
    let protocol = HttpProtocol;
    let iterations = 10_000_000;
    
    // Test dynamic dispatch
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(process_dynamic(&protocol as &dyn Protocol, &data));
    }
    let dynamic_time = start.elapsed();
    
    // Test static dispatch (zero-cost abstraction)
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(process_static(&protocol, &data));
    }
    let static_time = start.elapsed();
    
    // Direct call (baseline)
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(protocol.process(&data));
    }
    let direct_time = start.elapsed();
    
    println!("Performance Test Results ({} iterations):", iterations);
    println!("----------------------------------------");
    println!("Dynamic dispatch: {:?}", dynamic_time);
    println!("Static dispatch:  {:?}", static_time);
    println!("Direct call:      {:?}", direct_time);
    println!();
    
    let overhead_dynamic = ((dynamic_time.as_nanos() as f64 / direct_time.as_nanos() as f64) - 1.0) * 100.0;
    let overhead_static = ((static_time.as_nanos() as f64 / direct_time.as_nanos() as f64) - 1.0) * 100.0;
    
    println!("Overhead Analysis:");
    println!("Dynamic dispatch overhead: {:.1}%", overhead_dynamic);
    println!("Static dispatch overhead:  {:.1}%", overhead_static);
    
    if overhead_static.abs() < 5.0 {
        println!("✓ Zero-cost abstraction claim VERIFIED (< 5% overhead)");
    } else {
        println!("✗ Zero-cost abstraction claim NOT verified (> 5% overhead)");
    }
}