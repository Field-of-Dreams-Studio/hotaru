//! Ergonomic request accessors for [`PreferredLanguage`].

use hotaru_core::connection::TransportSpec;
use hotaru_http::context::HttpContext;

use super::parser::DEFAULT_FALLBACK_LANGUAGE;
use super::types::PreferredLanguage;

/// Convenience trait for downstream handlers that prefer method syntax over
/// direct `req.params.get::<PreferredLanguage>()` access.
pub trait PreferredLanguageRequestExt {
    /// Borrow language preferences installed by
    /// [`crate::PreferredLanguageMiddleware`].
    fn preferred_language(&self) -> Option<&PreferredLanguage>;

    /// Clone installed language preferences, or synthesize a default value when
    /// the middleware was not attached.
    fn preferred_language_or_default(&self) -> PreferredLanguage;
}

impl<TS: TransportSpec> PreferredLanguageRequestExt for HttpContext<TS> {
    fn preferred_language(&self) -> Option<&PreferredLanguage> {
        self.params.get::<PreferredLanguage>()
    }

    fn preferred_language_or_default(&self) -> PreferredLanguage {
        self.preferred_language().cloned().unwrap_or_else(|| {
            PreferredLanguage::from_accept_language(None, DEFAULT_FALLBACK_LANGUAGE)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_extension_reads_typed_params() {
        let mut req: HttpContext =
            HttpContext::new_client(String::new(), hotaru_http::safety::HttpSafety::default());

        assert!(req.preferred_language().is_none());
        assert_eq!(req.preferred_language_or_default().preferred(), "en");

        req.params
            .set(PreferredLanguage::from_accept_language(Some("zh-CN"), "en"));

        let preferred = req
            .preferred_language()
            .expect("typed params should contain PreferredLanguage");
        assert_eq!(preferred.preferred(), "zh-CN");
        assert_eq!(preferred.primary(), "zh");
    }
}
