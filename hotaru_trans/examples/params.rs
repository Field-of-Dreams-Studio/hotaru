//! Demonstrates the public `params!` and `params_clone!` macros.

use hotaru_core::extensions::{Params, ParamsClone};
use hotaru_trans::{params, params_clone};

struct RuntimeConfig {
    name: &'static str,
}

#[derive(Clone)]
struct SharedConfig {
    retries: usize,
}

fn main() {
    let params: Params = params!([
        RuntimeConfig { name: "local" },
        3_u8,
    ]);
    assert_eq!(params.get::<RuntimeConfig>().unwrap().name, "local");
    assert_eq!(params.get::<u8>(), Some(&3));

    let params: ParamsClone = params_clone!([
        SharedConfig { retries: 2 },
        String::from("shared"),
    ]);
    let cloned = params.clone();
    assert_eq!(cloned.get::<SharedConfig>().unwrap().retries, 2);
    assert_eq!(cloned.get::<String>().unwrap(), "shared");
}
