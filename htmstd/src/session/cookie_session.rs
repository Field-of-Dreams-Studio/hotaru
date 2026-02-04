use std::collections::HashMap;

use akari::Value;
use hotaru_core::app::middleware::AsyncMiddleware;
use hotaru_core::{connection::Protocol, http::traits::HTTP};
use hotaru_core::http::cookie::Cookie;
use hotaru_trans::middleware;

use hotaru_lib::ende::aes;

use crate::session::session_counter;

pub struct CSessionRW(HashMap<String, Value>, bool);

impl CSessionRW {
    pub fn new() -> Self {
        CSessionRW(HashMap::new(), false)
    }

    pub fn from_hash(map: HashMap<String, Value>) -> Self {
        CSessionRW(map, false)
    }

    pub fn insert(&mut self, key: String, value: Value) {
        self.0.insert(key, value);
        self.1 = true; // Mark as modified 
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        let removed = self.0.remove(key);
        if removed.is_some() {
            self.1 = true; // Mark as modified 
        }
        removed
    }

    pub fn is_modified(&self) -> bool {
        self.1
    }

    pub fn into_tuple(self) -> (Value, bool) {
        (Value::Dict(self.0), self.1)
    }
}

impl Default for CSessionRW {
    fn default() -> Self {
        CSessionRW(HashMap::new(), false)
    }
}

middleware!(
    pub CookieSession<HTTP> { 

        // println!("{:?}", req.get_cookies()); 
        let mut new_id_generated = false; 

        let session_id: u64 = match req
            .get_cookie_or_default("session_id")
            .get_value()
            .parse() { 
                Ok(id) => id,
                Err(_) => {
                    // If parsing fails, generate a new session ID 
                    new_id_generated = true; 
                    session_counter::generate_session_id() 
                } 
            }; 

        let serect_key = req
            .app()
            .and_then(|app| app.config.get::<String>().cloned())
            .unwrap_or("super_secret_key".to_string());
        let password = format!("{}{}", serect_key, session_id);
 
        let session_raw = req.get_cookie("session_cont").map(|c| c.get_value().to_owned()).unwrap_or("No Cookie Cont".to_owned()); 

        // println!("Session ID: {}, Session: {}", session_id, session_raw);

        let session = CSessionRW::from_hash(
            if let Value::Dict(map) = Value::from_json(
                &aes::decrypt(
                    &session_raw,
                    &password, 
                )
                .unwrap_or(String::from("Decrypt Error")),
            )
            .unwrap_or(Value::None)
            {
                map
            } else {
                HashMap::new()
            },
        );

        println!("CookieSession: Setting session in params");
        req.params.set(session);
        println!("CookieSession: About to call next middleware");
        let mut req = next(req).await; // Continue middleware chain
        println!("CookieSession: Returned from middleware chain"); 

        let (session, is_modified) = req
            .params
            .take::<CSessionRW>()
            .unwrap_or_default()
            .into_tuple();

        // println!("Cookie Session: {}", session);

        if is_modified|new_id_generated { 
            println!("Session modified, saving to cookies... {} ", session); 
            req.response = req
                .response
                .add_cookie("session_id", Cookie::new(session_id.to_string()).path("/"))
                .add_cookie(
                    "session_cont",
                    Cookie::new(
                        aes::encrypt(&session.into_json(), &password).unwrap_or("".to_string()),
                    )
                    .path("/"),
                ); // Set cookie with session ID 
        }

        req
    }
); 
