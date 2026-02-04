use dashmap::DashMap;
use hotaru_core::http::cookie::Cookie;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use lazy_static::lazy_static;
use tokio::time;

use hotaru_trans::middleware;
use hotaru_core::app::middleware::AsyncMiddleware;
use hotaru_core::{connection::Protocol, http::traits::HTTP};  

#[derive(Debug, Clone)]
pub struct SessionCont {
    pub expiry_time: u64,
    pub data: HashMap<String, String>,
}

lazy_static! {
    static ref SESSIONS: DashMap<u64, SessionCont> = DashMap::new();
} 

static DEFAULT_TTL: u64 = 3600 * 24 * 7; // Default TTL of 7 days  

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

fn generate_session_id() -> u64 {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time error")
        .as_millis() as u64;
    let counter = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed) & 0xFFFF;
    (timestamp << 16) | counter
}

pub fn new_session(initial_data: HashMap<String, String>, ttl_secs: u64) -> u64 {
    let id = generate_session_id();
    let expiry = SystemTime::now()
        .checked_add(Duration::from_secs(ttl_secs))
        .expect("Invalid TTL")
        .duration_since(UNIX_EPOCH)
        .expect("time error")
        .as_secs() as u64;

    let session = SessionCont {
        expiry_time: expiry,
        data: initial_data,
    };
    SESSIONS.insert(id, session);
    id
}

/// A lifetime-bound wrapper around a mutably borrowed session.
pub struct SessionRW<'a> {
    guard: dashmap::mapref::one::RefMut<'a, u64, SessionCont>,
    pub session_id: u64,
}

impl<'a> std::ops::Deref for SessionRW<'a> {
    type Target = SessionCont;
    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

impl<'a> std::ops::DerefMut for SessionRW<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.guard
    }
}

impl<'a> SessionRW<'a> {
    pub fn renew(&mut self, ttl_secs: u64) {
        self.touch(ttl_secs);
    }

    pub fn touch(&mut self, ttl_secs: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64;
        self.guard.expiry_time = now + ttl_secs;
    }

    pub fn get<T: AsRef<str>>(&self, key: T) -> Option<&String> {
        self.guard.data.get(key.as_ref())
    }

    pub fn set<T: Into<String>, U: Into<String>>(&mut self, key: T, value: U) {
        self.guard.data.insert(key.into(), value.into()); 
    }

    pub fn set_all(&mut self, data: HashMap<String, String>) {
        for (k, v) in data {
            self.guard.data.insert(k, v);
        }
    }
}

impl<'a> Default for SessionRW<'a> {
    fn default() -> Self {
        SessionRW {
            guard: SESSIONS.get_mut(&0).expect("Default session not found"),
            session_id: 0,
        } 
    }
} 

pub fn get_mut<'a>(id: u64) -> Result<SessionRW<'a>, &'static str> {
    match SESSIONS.get_mut(&id) {
        Some(guard) => Ok(SessionRW { guard, session_id: id }),
        None => Err("Session not found"),
    }
} 

middleware!(
    pub Session<HTTP>{ 
        let ttl = req.app()
            .and_then(|app| app.config.get::<u64>().cloned())
            .unwrap_or(DEFAULT_TTL); 
        let mut session_id: u64 = req.get_cookie_or_default("session_id")
            .get_value()
            .parse()
            .unwrap_or_else(|_| {
                new_session(HashMap::new(), ttl) 
            }); 
        let mut session = get_mut(session_id).unwrap_or_else(|_| { 
            session_id = new_session(HashMap::new(), ttl); 
            get_mut(session_id).unwrap() 
        }); 
        session.touch(ttl); // Refresh session expiration 
        req.params.set(session); 
        let mut req = next(req).await; // Continue middleware chain 
        req.response = req.response.add_cookie(
            "session_id", 
            Cookie::new(session_id.to_string()) 
                .path("/") 
        ); // Set cookie with session ID 
        req 
    } 
); 

async fn session_cleanup_task(interval_secs: u64) {
    let mut interval = time::interval(Duration::from_secs(interval_secs));
    loop {
        interval.tick().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64;
        SESSIONS.retain(|_, session| session.expiry_time > now);
    }
}

pub fn init_session_system() {
    tokio::spawn(session_cleanup_task(3600));
} 
