//! Configuration for [`crate::PreferredLanguageMiddleware`].

use super::parser::{DEFAULT_FALLBACK_LANGUAGE, normalize_fallback};

/// Runtime or endpoint configuration for [`crate::PreferredLanguageMiddleware`].
///
/// Store this in Hotaru config when the application wants a fallback other
/// than English. Endpoint config overrides runtime config, mirroring the
/// merge behavior of the other htmstd middleware (e.g. `Cors`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreferredLanguageSettings {
    fallback: String,
}

impl PreferredLanguageSettings {
    /// Create settings with a specific fallback language tag.
    pub fn new<T: Into<String>>(fallback: T) -> Self {
        Self {
            fallback: normalize_fallback(fallback.into()),
        }
    }

    /// Builder-style setter for the fallback language tag.
    pub fn fallback<T: Into<String>>(mut self, fallback: T) -> Self {
        self.fallback = normalize_fallback(fallback.into());
        self
    }

    /// The language used when the request does not carry an acceptable
    /// `Accept-Language` value.
    pub fn fallback_language(&self) -> &str {
        &self.fallback
    }

    /// Merge runtime settings with endpoint settings.
    ///
    /// Currently only the fallback language is configurable, so a non-empty
    /// `other` fully overrides `self`.
    pub fn merge(&self, other: &Self) -> Self {
        if other.fallback.is_empty() {
            self.clone()
        } else {
            other.clone()
        }
    }
}

impl Default for PreferredLanguageSettings {
    fn default() -> Self {
        Self::new(DEFAULT_FALLBACK_LANGUAGE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_merge_prefers_non_empty_override() {
        let runtime = PreferredLanguageSettings::new("en");
        let endpoint = PreferredLanguageSettings::new("zh-CN");

        assert_eq!(runtime.merge(&endpoint).fallback_language(), "zh-CN");
        // Default fallback normalizes empty input back to "en".
        assert_eq!(
            PreferredLanguageSettings::default().fallback_language(),
            "en"
        );
    }

    #[test]
    fn builder_and_new_normalize_blank_fallback() {
        assert_eq!(
            PreferredLanguageSettings::new("   ").fallback_language(),
            "en"
        );
        assert_eq!(
            PreferredLanguageSettings::default()
                .fallback("  ja  ")
                .fallback_language(),
            "ja"
        );
    }
}
