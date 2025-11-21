// pub mod traits_bench;
// pub mod memory_bench;
// pub mod load_test;

// use std::time::{Duration, Instant};

// /// Helper to measure operation time
// pub fn measure_time<F, R>(f: F) -> (R, Duration)
// where
//     F: FnOnce() -> R,
// {
//     let start = Instant::now();
//     let result = f();
//     (result, start.elapsed())
// }

// /// Statistics for benchmark results
// #[derive(Debug, Clone)]
// pub struct BenchStats {
//     pub mean: f64,
//     pub median: f64,
//     pub p95: f64,
//     pub p99: f64,
//     pub min: f64,
//     pub max: f64,
//     pub std_dev: f64,
// }

// impl BenchStats {
//     pub fn from_samples(mut samples: Vec<f64>) -> Self {
//         samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
//         let len = samples.len();
        
//         let mean = samples.iter().sum::<f64>() / len as f64;
//         let median = samples[len / 2];
//         let p95 = samples[(len as f64 * 0.95) as usize];
//         let p99 = samples[(len as f64 * 0.99) as usize];
//         let min = samples[0];
//         let max = samples[len - 1];
        
//         let variance = samples.iter()
//             .map(|x| (x - mean).powi(2))
//             .sum::<f64>() / len as f64;
//         let std_dev = variance.sqrt();
        
//         BenchStats {
//             mean,
//             median,
//             p95,
//             p99,
//             min,
//             max,
//             std_dev,
//         }
//     }
// }
