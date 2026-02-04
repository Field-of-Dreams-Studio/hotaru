//! Demonstration of the connection pool functionality
//!
//! This example shows how the connection pool works by making multiple
//! sequential HTTP requests to the same host and displaying pool statistics.

use hotaru_core::client::pool::ConnectionPool;
use hotaru_core::http::context::HttpContext;
use hotaru_core::http::request::request_templates::get_request;
use hotaru_core::http::safety::HttpSafety;

#[tokio::main]
async fn main() {
    println!("Connection Pool Demo");
    println!("====================\n");

    // Make multiple requests to the same host
    let host = "http://httpbin.org";
    let paths = vec!["/get", "/headers", "/user-agent", "/delay/1"];

    println!("Making {} sequential requests to {}", paths.len(), host);
    println!("This demonstrates connection reuse via the connection pool.\n");

    for (i, path) in paths.iter().enumerate() {
        println!("Request {} to {}...", i + 1, path);

        let request = get_request(*path);

        match HttpContext::send_request(host, request, HttpSafety::default()).await {
            Ok(response) => {
                println!("  ✓ Status: {}", response.meta.start_line.status_code());

                // Show pool statistics after each request
                let stats = ConnectionPool::global().stats();
                println!("  Pool stats: hits={}, misses={}, pooled={}",
                    stats.hits, stats.misses, stats.pooled_connections);
            }
            Err(e) => {
                println!("  ✗ Error: {:?}", e);
            }
        }
        println!();
    }

    println!("\nFinal Pool Statistics");
    println!("====================");
    let final_stats = ConnectionPool::global().stats();
    println!("Total hits:              {}", final_stats.hits);
    println!("Total misses:            {}", final_stats.misses);
    println!("Pooled connections:      {}", final_stats.pooled_connections);
    println!("Evictions:               {}", final_stats.evictions);

    if final_stats.hits > 0 {
        let hit_rate = (final_stats.hits as f64 / (final_stats.hits + final_stats.misses) as f64) * 100.0;
        println!("Hit rate:                {:.1}%", hit_rate);
        println!("\n✓ Connection pooling is working! Connections were reused.");
    } else {
        println!("\n⚠ No pool hits detected. All requests created new connections.");
    }
}
