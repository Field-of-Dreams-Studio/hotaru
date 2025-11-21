use std::{sync::atomic::{AtomicU64, Ordering}, time::{SystemTime, UNIX_EPOCH}};

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0); 

pub fn generate_session_id() -> u64 {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time error")
        .as_millis() as u64;
    let counter = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed) & 0xFFFF;
    (timestamp << 16) | counter
} 
