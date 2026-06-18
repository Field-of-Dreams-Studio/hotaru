use std::collections::HashMap;
use std::sync::OnceLock;

use akari::Value;
use hotaru_core::executable::middleware::AsyncMiddleware;
use hotaru_core::{debug_log, debug_warn};
use hotaru_http::cookie::{Cookie, SameSite};
use hotaru_core::protocol::{Protocol, RequestContext};
use hotaru_http::traits::HTTP;
use hotaru_trans::middleware;

use hotaru_lib::ende::aes;

use crate::session::session_counter;

/// Process-wide fallback session secret, generated once on first use when
/// no usable [`SessionSecret`] is registered in the runtime config.
///
/// A random secret keeps session cookies unforgeable even when the app
/// forgot to configure one, but it lives only as long as the process:
/// sessions are invalidated on restart and cannot be shared between
/// instances behind a load balancer. Register a real secret in production.
static FALLBACK_SECRET: OnceLock<SessionSecret> = OnceLock::new();

fn fallback_secret() -> String {
    FALLBACK_SECRET
        .get_or_init(|| {
            debug_warn!(
                "CookieSession: no session secret configured; \
                 using a random per-process secret. Sessions will not \
                 survive restarts or be shared across instances."
            );
            SessionSecret::generate()
        })
        .0
        .clone()
        .expect("SessionSecret::generate always holds a secret")
}

/// Minimum secret length in bytes. The key derivation (HKDF) does no
/// brute-force stretching, so the secret itself must carry the entropy.
const MIN_SECRET_BYTES: usize = 32;

/// Newtype for the cookie-session secret. Register with
/// `set_config(SessionSecret::new(..))` so an unrelated `String` config
/// can never silently become key material.
#[derive(Clone)]
pub struct SessionSecret(Option<String>);

impl SessionSecret {
    /// Secrets shorter than 32 bytes are refused: the value is dropped and
    /// the random per-process fallback is used instead. Use 32+ random
    /// bytes, not a passphrase — length is only a proxy for entropy.
    pub fn new<T: Into<String>>(secret: T) -> Self {
        let secret = secret.into();
        if secret.len() < MIN_SECRET_BYTES {
            debug_warn!(
                "SessionSecret: secret shorter than {} bytes refused; \
                 the random per-process fallback will be used.",
                MIN_SECRET_BYTES
            );
            SessionSecret(None)
        } else {
            SessionSecret(Some(secret))
        }
    }

    /// A fresh random secret (64 printable ASCII chars, ~419 bits).
    pub fn generate() -> Self {
        SessionSecret(Some(hotaru_lib::random::random_string(64)))
    }
}

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
            .runtime()
            .and_then(|rt| rt.get_config::<SessionSecret>())
            .and_then(|s| s.0)
            .unwrap_or_else(fallback_secret);
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

        debug_log!("CookieSession: Setting session in params");
        req.params.set(session);
        debug_log!("CookieSession: About to call next middleware");
        let mut req = next(req).await?; // Continue middleware chain
        debug_log!("CookieSession: Returned from middleware chain");

        let (session, is_modified) = req
            .params
            .take::<CSessionRW>()
            .unwrap_or_default()
            .into_tuple();

        // println!("Cookie Session: {}", session);

        if is_modified|new_id_generated {
            // Never log `session` itself: it holds decrypted user data.
            debug_log!("CookieSession: session modified, saving to cookies (id={})", session_id);
            req.response = req
                .response
                .add_cookie(
                    "session_id",
                    Cookie::new(session_id.to_string())
                        .path("/")
                        .http_only(true)
                        .secure(true)
                        .same_site(SameSite::Lax),
                )
                .add_cookie(
                    "session_cont",
                    Cookie::new(
                        aes::encrypt(&session.into_json(), &password).unwrap_or("".to_string()),
                    )
                    .path("/")
                    .http_only(true)
                    .secure(true)
                    .same_site(SameSite::Lax),
                ); // Set cookie with session ID
        }

        Ok(req)
    }
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_secret_refused() {
        assert!(SessionSecret::new("too short").0.is_none());
        assert!(SessionSecret::new("a".repeat(MIN_SECRET_BYTES - 1)).0.is_none());
    }

    #[test]
    fn long_secret_accepted() {
        assert!(SessionSecret::new("a".repeat(MIN_SECRET_BYTES)).0.is_some());
    }

    #[test]
    fn generated_secret_is_long_enough() {
        let secret = SessionSecret::generate();
        assert!(secret.0.expect("generated").len() >= MIN_SECRET_BYTES);
    }
}
