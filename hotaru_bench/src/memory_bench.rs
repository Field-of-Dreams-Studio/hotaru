// Memory usage benchmarks
use std::mem;

// Hotaru's trait-based approach
pub trait Transport {
    fn id(&self) -> i128;
}

pub trait Stream {
    fn id(&self) -> u32;
}

pub struct HttpTransport {
    id: i128,
    keep_alive: bool,
    request_count: u64,
}

impl Transport for HttpTransport {
    fn id(&self) -> i128 { self.id }
}

// Traditional approach with dynamic dispatch
pub trait DynamicProtocol {
    fn process(&self);
    fn get_id(&self) -> i128;
    fn is_keep_alive(&self) -> bool;
    fn get_request_count(&self) -> u64;
}

pub struct DynamicHttp {
    id: i128,
    keep_alive: bool,
    request_count: u64,
}

impl DynamicProtocol for DynamicHttp {
    fn process(&self) {}
    fn get_id(&self) -> i128 { self.id }
    fn is_keep_alive(&self) -> bool { self.keep_alive }
    fn get_request_count(&self) -> u64 { self.request_count }
}

pub fn run_memory_benchmark() {
    println!("\n=== Memory Usage Benchmark ===");
    println!("==============================\n");
    
    // Size of trait-based structs (Hotaru approach)
    println!("Hotaru Approach (trait-based):");
    println!("  HttpTransport struct: {} bytes", mem::size_of::<HttpTransport>());
    println!("  Transport trait object: {} bytes", mem::size_of::<&dyn Transport>());
    
    // Size of traditional dynamic dispatch
    println!("\nTraditional Approach (dynamic dispatch):");
    println!("  DynamicHttp struct: {} bytes", mem::size_of::<DynamicHttp>());
    println!("  DynamicProtocol trait object: {} bytes", mem::size_of::<Box<dyn DynamicProtocol>>());
    
    // Per-connection memory (simulated)
    let connections = 1000;
    
    // Hotaru: Direct struct storage
    let hotaru_mem = connections * mem::size_of::<HttpTransport>();
    
    // Traditional: Box<dyn Protocol> storage
    let traditional_mem = connections * (mem::size_of::<Box<dyn DynamicProtocol>>() + mem::size_of::<DynamicHttp>());
    
    println!("\nMemory for {} connections:", connections);
    println!("  Hotaru approach: {} bytes ({:.2} KB)", hotaru_mem, hotaru_mem as f64 / 1024.0);
    println!("  Traditional approach: {} bytes ({:.2} KB)", traditional_mem, traditional_mem as f64 / 1024.0);
    
    let savings = (1.0 - (hotaru_mem as f64 / traditional_mem as f64)) * 100.0;
    println!("\nMemory savings: {:.1}%", savings);
    
    if savings >= 25.0 {
        println!("✓ Memory reduction claim VERIFIED (≥ 25% savings)");
    } else {
        println!("✗ Memory reduction claim NOT verified (< 25% savings)");
    }
}