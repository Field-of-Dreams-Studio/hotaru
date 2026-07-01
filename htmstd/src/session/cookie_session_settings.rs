use hotaru_core::app::common::RunMode;
use hotaru_http::cookie::{Cookie, SameSite};

/// Policy for the `Secure` attribute on cookies written by [`CookieSession`](super::CookieSession).
///
/// `Secure` cookies are not sent by browsers over plain HTTP. That is the
/// correct production default, but local/plain-HTTP deployments may need to
/// opt out explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookieSecurity {
    /// Always mark session cookies as `Secure`.
    Secure,
    /// Never mark session cookies as `Secure`.
    ///
    /// Use this only for local development or trusted plain-HTTP environments.
    Insecure,
    /// Resolve from the app runtime mode.
    ///
    /// `Production` and `Beta` are secure; `Development` and `Build` are
    /// insecure so local HTTP sessions work without changing code.
    Auto,
}

impl CookieSecurity {
    /// Resolve this policy to the concrete value passed to [`Cookie::secure`].
    pub fn resolve(self, mode: RunMode) -> bool {
        match self {
            Self::Secure => true,
            Self::Insecure => false,
            Self::Auto => matches!(mode, RunMode::Production | RunMode::Beta),
        }
    }
}

impl Default for CookieSecurity {
    fn default() -> Self {
        Self::Secure
    }
}

/// Cookie attributes used by the encrypted cookie-session middleware.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CookieSessionSettings {
    /// Policy for the `Secure` cookie attribute.
    pub security: CookieSecurity,
    /// `SameSite` policy for session cookies.
    pub same_site: SameSite,
    /// Path applied to session cookies.
    pub path: String,
    /// Whether session cookies should be inaccessible to JavaScript.
    pub http_only: bool,
}

impl CookieSessionSettings {
    /// Create default, production-safe cookie-session settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Use the given `Secure` policy.
    pub fn security(mut self, security: CookieSecurity) -> Self {
        self.security = security;
        self
    }

    /// Always set the `Secure` attribute.
    pub fn secure(self) -> Self {
        self.security(CookieSecurity::Secure)
    }

    /// Never set the `Secure` attribute.
    pub fn insecure(self) -> Self {
        self.security(CookieSecurity::Insecure)
    }

    /// Resolve `Secure` from app [`RunMode`].
    pub fn auto_security(self) -> Self {
        self.security(CookieSecurity::Auto)
    }

    /// Set the `SameSite` cookie attribute.
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = same_site;
        self
    }

    /// Set the cookie path.
    pub fn path<T: Into<String>>(mut self, path: T) -> Self {
        self.path = path.into();
        self
    }

    /// Set the `HttpOnly` cookie attribute.
    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    /// Resolve the configured `Secure` policy for a concrete app mode.
    pub fn resolve_secure(&self, mode: RunMode) -> bool {
        self.security.resolve(mode)
    }

    /// Apply these settings to a cookie value.
    pub fn apply_to_cookie(&self, cookie: Cookie, mode: RunMode) -> Cookie {
        cookie
            .path(&self.path)
            .http_only(self.http_only)
            .secure(self.resolve_secure(mode))
            .same_site(self.same_site)
    }
}

impl Default for CookieSessionSettings {
    fn default() -> Self {
        Self {
            security: CookieSecurity::Secure,
            same_site: SameSite::Lax,
            path: "/".to_string(),
            http_only: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_preserve_existing_secure_behavior() {
        let settings = CookieSessionSettings::default();

        assert_eq!(settings.security, CookieSecurity::Secure);
        assert_eq!(settings.same_site, SameSite::Lax);
        assert_eq!(settings.path, "/");
        assert!(settings.http_only);
        assert!(settings.resolve_secure(RunMode::Development));
    }

    #[test]
    fn security_policy_resolves_from_mode() {
        assert!(CookieSecurity::Secure.resolve(RunMode::Development));
        assert!(!CookieSecurity::Insecure.resolve(RunMode::Production));

        assert!(CookieSecurity::Auto.resolve(RunMode::Production));
        assert!(CookieSecurity::Auto.resolve(RunMode::Beta));
        assert!(!CookieSecurity::Auto.resolve(RunMode::Development));
        assert!(!CookieSecurity::Auto.resolve(RunMode::Build));
    }

    #[test]
    fn builder_updates_cookie_attributes() {
        let settings = CookieSessionSettings::new()
            .insecure()
            .same_site(SameSite::Strict)
            .path("/app")
            .http_only(false);

        assert_eq!(settings.security, CookieSecurity::Insecure);
        assert_eq!(settings.same_site, SameSite::Strict);
        assert_eq!(settings.path, "/app");
        assert!(!settings.http_only);
    }

    #[test]
    fn apply_to_cookie_sets_all_attributes() {
        let cookie = CookieSessionSettings::new()
            .auto_security()
            .same_site(SameSite::None)
            .path("/session")
            .apply_to_cookie(Cookie::new("value"), RunMode::Production);

        assert_eq!(cookie.get_path(), Some("/session".to_string()));
        assert_eq!(cookie.get_http_only(), Some(true));
        assert_eq!(cookie.get_secure(), Some(true));
        assert_eq!(cookie.get_same_site(), Some(SameSite::None));
    }
}
