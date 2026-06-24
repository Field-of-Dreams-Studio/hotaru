//! Core data model for parsed `Accept-Language` preferences.
//!
//! [`PreferredLanguage`] is the struct that [`crate::PreferredLanguageMiddleware`]
//! stores in `req.params`, and [`LanguageRange`] is one parsed item of it.

use super::parser::{
    DEFAULT_FALLBACK_LANGUAGE, language_match_specificity, language_matches, normalize_fallback,
    parse_accept_language, primary_subtag,
};

/// Maximum HTTP quality value expressed in q-millis (`q=1.000`).
///
/// HTTP `Accept-Language` q-values have at most three decimal places
/// (RFC 9110 §12.4.2), so the full precision fits in a `u16` as an integer
/// number of thousandths. Storing quality as `u16` q-millis avoids float
/// equality/ordering pitfalls entirely.
pub const MAX_QUALITY_MILLIS: u16 = 1000;

/// One parsed item from an HTTP `Accept-Language` header.
///
/// Quality is stored as a `u16` in q-millis (thousandths), so `q=1` is
/// `1000`, `q=0.8` is `800`, and `q=0` is `0`. This keeps comparisons exact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageRange {
    tag: String,
    quality_millis: u16,
    order: usize,
}

impl LanguageRange {
    /// Construct a range directly from a tag and a `u16` q-millis value.
    ///
    /// The quality is clamped to `0..=MAX_QUALITY_MILLIS`.
    pub fn new<T: Into<String>>(tag: T, quality_millis: u16, order: usize) -> Self {
        Self {
            tag: tag.into(),
            quality_millis: quality_millis.min(MAX_QUALITY_MILLIS),
            order,
        }
    }

    /// Construct a range without clamping. Used by the parser, which already
    /// guarantees `quality_millis <= MAX_QUALITY_MILLIS`.
    pub(crate) fn from_parts(tag: String, quality_millis: u16, order: usize) -> Self {
        Self {
            tag,
            quality_millis,
            order,
        }
    }

    /// The original language tag/range, such as `en-US`, `zh-Hant`, or `*`.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// The quality value in q-millis (thousandths). `q=1` → `1000`.
    pub fn quality_millis(&self) -> u16 {
        self.quality_millis
    }

    /// The quality value as a floating-point q-value in `0.0..=1.0`.
    ///
    /// This is a convenience view; the canonical representation is
    /// [`Self::quality_millis`].
    pub fn quality(&self) -> f32 {
        f32::from(self.quality_millis) / f32::from(MAX_QUALITY_MILLIS)
    }

    /// Original order in the header. Lower values appeared earlier.
    pub fn order(&self) -> usize {
        self.order
    }

    /// A range is acceptable when it has a non-zero quality and a tag.
    fn is_acceptable(&self) -> bool {
        self.quality_millis > 0 && !self.tag.is_empty()
    }
}

/// Parsed user language preferences for the current request.
///
/// [`crate::PreferredLanguageMiddleware`] stores one instance in `req.params`,
/// so downstream middleware and handlers can read it as:
///
/// ```ignore
/// let lang = req.params.get::<PreferredLanguage>().unwrap();
/// let template_lang = lang
///     .best_match(["en", "zh-CN", "ja"].iter().copied())
///     .unwrap_or("en");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreferredLanguage {
    raw: Option<String>,
    ranges: Vec<LanguageRange>,
    fallback: String,
}

impl PreferredLanguage {
    /// Parse an `Accept-Language` header using the default fallback (`en`).
    pub fn parse<T: AsRef<str>>(header: T) -> Self {
        Self::from_accept_language(Some(header.as_ref()), DEFAULT_FALLBACK_LANGUAGE)
    }

    /// Build preferences from an optional `Accept-Language` header value.
    pub fn from_accept_language(header: Option<&str>, fallback: impl Into<String>) -> Self {
        let fallback = normalize_fallback(fallback.into());
        let raw = header
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let ranges = raw
            .as_deref()
            .map(parse_accept_language)
            .unwrap_or_default();

        Self {
            raw,
            ranges,
            fallback,
        }
    }

    /// The exact raw `Accept-Language` header value, if present.
    pub fn raw(&self) -> Option<&str> {
        self.raw.as_deref()
    }

    /// All parsed ranges in original header order, including `q=0` ranges.
    pub fn ranges(&self) -> &[LanguageRange] {
        &self.ranges
    }

    /// Returns `true` when no usable (non-zero quality) language ranges exist.
    pub fn is_empty(&self) -> bool {
        self.ranges.iter().all(|range| !range.is_acceptable())
    }

    /// The configured fallback language.
    pub fn fallback(&self) -> &str {
        &self.fallback
    }

    /// The most preferred concrete language from the header, or the fallback.
    ///
    /// Wildcard (`*`) ranges are never returned as a concrete language; if the
    /// header contains only `*`, this returns the fallback.
    pub fn preferred(&self) -> &str {
        self.ranges
            .iter()
            .filter(|range| range.is_acceptable() && range.tag != "*")
            .max_by(|left, right| {
                left.quality_millis
                    .cmp(&right.quality_millis)
                    .then_with(|| right.order.cmp(&left.order))
            })
            .map(|range| range.tag.as_str())
            .unwrap_or(self.fallback.as_str())
    }

    /// The primary subtag of [`Self::preferred`], e.g. `en` for `en-US`.
    pub fn primary(&self) -> &str {
        primary_subtag(self.preferred())
    }

    /// The q-millis quality for a candidate language.
    ///
    /// Considers exact, primary-subtag, more-specific, and wildcard matches,
    /// preferring the most specific matching range, then highest quality, then
    /// earliest header position. Returns `0` when nothing matches.
    pub fn quality_millis_for(&self, language: &str) -> u16 {
        self.ranges
            .iter()
            .filter(|range| language_matches(&range.tag, language))
            .max_by(|left, right| {
                language_match_specificity(&left.tag, language)
                    .cmp(&language_match_specificity(&right.tag, language))
                    .then_with(|| left.quality_millis.cmp(&right.quality_millis))
                    .then_with(|| right.order.cmp(&left.order))
            })
            .map(|range| range.quality_millis)
            .unwrap_or(0)
    }

    /// The floating-point q-value for a candidate language (`0.0` when none).
    pub fn quality_for(&self, language: &str) -> f32 {
        f32::from(self.quality_millis_for(language)) / f32::from(MAX_QUALITY_MILLIS)
    }

    /// Returns whether a candidate language is acceptable to the user.
    pub fn accepts(&self, language: &str) -> bool {
        self.quality_millis_for(language) > 0
    }

    /// Pick the best language from a supported list according to this request.
    ///
    /// User preference (quality, then header order) wins first; when a user
    /// range matches multiple supported languages, the supported-list order is
    /// the tie-breaker. An explicit `q=0` always rejects a language even if a
    /// wildcard would otherwise allow it.
    pub fn best_match<'a, I>(&self, supported: I) -> Option<&'a str>
    where
        I: IntoIterator<Item = &'a str>,
    {
        let supported: Vec<&'a str> = supported.into_iter().collect();
        if supported.is_empty() {
            return None;
        }

        let mut ranges = self
            .ranges
            .iter()
            .filter(|range| range.is_acceptable())
            .collect::<Vec<_>>();
        ranges.sort_by(|left, right| {
            right
                .quality_millis
                .cmp(&left.quality_millis)
                .then_with(|| left.order.cmp(&right.order))
        });

        for range in ranges {
            if let Some(language) = supported
                .iter()
                .copied()
                .find(|language| language_matches(&range.tag, language) && self.accepts(language))
            {
                return Some(language);
            }
        }

        supported
            .iter()
            .copied()
            .find(|language| language.eq_ignore_ascii_case(&self.fallback))
    }

    /// Owned-string convenience wrapper around [`Self::best_match`].
    pub fn best_match_owned<I, S>(&self, supported: I) -> Option<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let supported: Vec<String> = supported
            .into_iter()
            .map(|language| language.as_ref().to_string())
            .collect();
        self.best_match(supported.iter().map(String::as_str))
            .map(ToOwned::to_owned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_preference_order_and_quality_as_millis() {
        let language = PreferredLanguage::parse("fr-CA, fr;q=0.8, en-US;q=0.6, en;q=0.4");

        assert_eq!(
            language.raw(),
            Some("fr-CA, fr;q=0.8, en-US;q=0.6, en;q=0.4")
        );
        assert_eq!(language.preferred(), "fr-CA");
        assert_eq!(language.primary(), "fr");
        assert_eq!(language.ranges().len(), 4);

        // q-millis are exact integers.
        assert_eq!(language.ranges()[0].quality_millis(), 1000);
        assert_eq!(language.ranges()[1].quality_millis(), 800);
        assert_eq!(language.ranges()[2].quality_millis(), 600);
        assert_eq!(language.ranges()[3].quality_millis(), 400);

        assert_eq!(language.quality_millis_for("en-US"), 600);
        assert!((language.quality_for("en-US") - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn falls_back_when_header_missing() {
        let missing = PreferredLanguage::from_accept_language(None, "zh-CN");

        assert!(missing.is_empty());
        assert_eq!(missing.raw(), None);
        assert_eq!(missing.preferred(), "zh-CN");
        assert_eq!(missing.primary(), "zh");
        assert_eq!(missing.fallback(), "zh-CN");
    }

    #[test]
    fn empty_or_whitespace_header_is_treated_as_missing() {
        let blank = PreferredLanguage::from_accept_language(Some("   "), "en");
        assert!(blank.is_empty());
        assert_eq!(blank.raw(), None);
        assert_eq!(blank.preferred(), "en");
    }

    #[test]
    fn falls_back_when_all_weights_zero() {
        let language = PreferredLanguage::from_accept_language(Some("fr;q=0, en;q=0"), "en");

        assert!(language.is_empty());
        assert_eq!(language.preferred(), "en");
        assert!(!language.accepts("fr"));
        assert!(!language.accepts("en"));
    }

    #[test]
    fn negotiates_best_supported_language_by_quality() {
        let language = PreferredLanguage::parse("fr-CA, zh-CN;q=0.9, en;q=0.8");

        assert_eq!(
            language.best_match(["en", "zh-CN", "fr"].iter().copied()),
            Some("fr")
        );
        assert_eq!(
            language.best_match(["en", "zh-HK", "zh-CN"].iter().copied()),
            Some("zh-CN")
        );
    }

    #[test]
    fn best_match_uses_supported_order_as_tie_breaker() {
        // Both "en-US" and "en-GB" match the user's "en" range equally; the
        // supported-list order decides.
        let language = PreferredLanguage::parse("en");

        assert_eq!(
            language.best_match(["en-GB", "en-US"].iter().copied()),
            Some("en-GB")
        );
        assert_eq!(
            language.best_match(["en-US", "en-GB"].iter().copied()),
            Some("en-US")
        );
    }

    #[test]
    fn best_match_returns_none_for_empty_supported_list() {
        let language = PreferredLanguage::parse("en");
        let empty: [&str; 0] = [];
        assert_eq!(language.best_match(empty.iter().copied()), None);
    }

    #[test]
    fn best_match_falls_back_to_supported_fallback_when_nothing_matches() {
        let language = PreferredLanguage::from_accept_language(Some("fr"), "en");

        // "fr" is preferred but unsupported; the fallback "en" is supported.
        assert_eq!(
            language.best_match(["en", "de"].iter().copied()),
            Some("en")
        );
        // Fallback not in the supported list → no match.
        assert_eq!(language.best_match(["de", "es"].iter().copied()), None);
    }

    #[test]
    fn zh_tw_can_display_any_configured_supported_zh_script_fallback() {
        let language = PreferredLanguage::from_accept_language(Some("zh-TW"), "zh-Hant");

        // We intentionally do not encode Chinese region/script alias rules here.
        // If the app supports zh scripts and configures one as fallback, the
        // negotiation can still display a supported zh script.
        assert_eq!(
            language.best_match(["zh-Hans", "zh-Hant"].iter().copied()),
            Some("zh-Hant")
        );
    }

    #[test]
    fn wildcard_matches_supported_language() {
        let language = PreferredLanguage::parse("*;q=0.5");

        // Wildcard never resolves to a concrete preferred language.
        assert_eq!(language.preferred(), "en");
        assert_eq!(
            language.best_match(["ja", "en"].iter().copied()),
            Some("ja")
        );
        assert!(language.accepts("anything"));
        assert_eq!(language.quality_millis_for("anything"), 500);
    }

    #[test]
    fn explicit_zero_quality_overrides_wildcard() {
        let language = PreferredLanguage::parse("fr;q=0, *;q=0.5");

        assert!(!language.accepts("fr"));
        assert!(language.accepts("de"));
        assert_eq!(
            language.best_match(["fr", "de"].iter().copied()),
            Some("de")
        );
    }

    #[test]
    fn matching_is_case_insensitive_and_subtag_aware() {
        let language = PreferredLanguage::parse("EN-us");

        assert!(language.accepts("en-US"));
        assert!(language.accepts("en"));
        assert!(language.accepts("EN"));
        assert_eq!(language.quality_millis_for("en-US"), 1000);
        // "en" range matches more specific "en-US" supported tag.
        assert_eq!(
            language.best_match(["fr", "en"].iter().copied()),
            Some("en")
        );
    }

    #[test]
    fn more_specific_range_wins_quality_lookup() {
        // "en" at 0.4 and "en-US" at 0.9: querying "en-US" should pick the
        // more specific (en-US) range, not merely the highest-quality match.
        let language = PreferredLanguage::parse("en;q=0.4, en-US;q=0.9");

        assert_eq!(language.quality_millis_for("en-US"), 900);
        assert_eq!(language.quality_millis_for("en"), 400);
    }

    #[test]
    fn owned_best_match_accepts_owned_lists() {
        let language = PreferredLanguage::parse("es-MX, en;q=0.8");
        let supported = vec!["en".to_string(), "es".to_string()];

        assert_eq!(language.best_match_owned(supported), Some("es".to_string()));
    }

    #[test]
    fn language_range_new_clamps_quality_to_max() {
        let range = LanguageRange::new("en", 5000, 0);
        assert_eq!(range.quality_millis(), MAX_QUALITY_MILLIS);
        assert!((range.quality() - 1.0).abs() < f32::EPSILON);
    }
}
