//! Pure parsing and language-matching helpers for `Accept-Language`.
//!
//! Everything here is side-effect free and independent of Hotaru request
//! types, which keeps it easy to unit-test in isolation.

use hotaru_core::debug_log;

use super::types::{LanguageRange, MAX_QUALITY_MILLIS};

/// Default language used when the request carries no acceptable preference.
pub(crate) const DEFAULT_FALLBACK_LANGUAGE: &str = "en";

/// Upper bound on the number of language ranges parsed from a single header.
///
/// Real clients send only a handful of ranges; this cap stops a pathological
/// `Accept-Language` (thousands of comma-separated ranges) from feeding the
/// downstream matching helpers a work-list large enough to become an
/// algorithmic-complexity DoS. Ranges beyond the cap are dropped, keeping the
/// earliest ones — i.e. the client's stated preference order.
pub(crate) const MAX_LANGUAGE_RANGES: usize = 64;

/// Trim a fallback tag and substitute the default when it is empty.
pub(crate) fn normalize_fallback(mut fallback: String) -> String {
    fallback = fallback.trim().to_string();
    if fallback.is_empty() {
        DEFAULT_FALLBACK_LANGUAGE.to_string()
    } else {
        fallback
    }
}

/// Parse a full `Accept-Language` header into ordered [`LanguageRange`]s.
///
/// At most [`MAX_LANGUAGE_RANGES`] ranges are returned; any excess is dropped
/// and logged, so a hostile header cannot inflate downstream matching cost.
pub(crate) fn parse_accept_language(header: &str) -> Vec<LanguageRange> {
    let mut items = header
        .split(',')
        .enumerate()
        .filter_map(|(order, item)| parse_language_range(item, order));

    let ranges: Vec<LanguageRange> = items.by_ref().take(MAX_LANGUAGE_RANGES).collect();

    // Pull one more only to detect (and log) truncation; the remaining ranges
    // are never parsed, so this stays linear in the header length.
    if items.next().is_some() {
        debug_log!(
            "PreferredLanguage: Accept-Language exceeded {} ranges; excess dropped",
            MAX_LANGUAGE_RANGES
        );
    }

    ranges
}

/// Parse a single `tag;q=value` item. Returns `None` for empty tags.
fn parse_language_range(item: &str, order: usize) -> Option<LanguageRange> {
    let mut parts = item.split(';');
    let tag = parts.next()?.trim();
    if tag.is_empty() {
        return None;
    }

    let mut quality_millis = MAX_QUALITY_MILLIS;
    for param in parts {
        let mut key_value = param.splitn(2, '=');
        let key = key_value.next().unwrap_or_default().trim();
        let value = key_value.next().unwrap_or_default().trim();
        if key.eq_ignore_ascii_case("q") {
            quality_millis = parse_quality_millis(value);
            break;
        }
    }

    Some(LanguageRange::from_parts(
        tag.to_string(),
        quality_millis,
        order,
    ))
}

/// Parse an HTTP q-value (`0`..`1` with up to three decimals) into q-millis.
///
/// Invalid values are treated as `0` (rejected), matching the conservative
/// reading that a malformed weight should not silently grant preference.
pub(crate) fn parse_quality_millis(value: &str) -> u16 {
    let value = value.trim();
    if value.is_empty() {
        return 0;
    }

    let (integer_part, fraction_part) = match value.split_once('.') {
        Some((integer, fraction)) => {
            if fraction.len() > 3 || fraction.bytes().any(|byte| !byte.is_ascii_digit()) {
                return 0;
            }
            (integer, fraction)
        }
        None => (value, ""),
    };

    match integer_part {
        "0" => {}
        "1" => {
            if fraction_part.bytes().all(|byte| byte == b'0') {
                return MAX_QUALITY_MILLIS;
            }
            return 0;
        }
        _ => return 0,
    }

    // Right-pad valid fractional digits to thousandths.
    let mut millis: u16 = 0;
    for index in 0..3 {
        millis *= 10;
        if let Some(byte) = fraction_part.as_bytes().get(index) {
            millis += u16::from(byte - b'0');
        }
    }

    millis
}

/// Whether a header `range` matches a candidate `language`.
///
/// Matching is case-insensitive and subtag-aware in both directions, and the
/// `*` wildcard matches everything.
pub(crate) fn language_matches(range: &str, language: &str) -> bool {
    let range = range.trim();
    let language = language.trim();

    if range == "*" {
        return true;
    }

    if range.eq_ignore_ascii_case(language) {
        return true;
    }

    starts_with_language_boundary(language, range) || starts_with_language_boundary(range, language)
}

/// Whether `value` begins with `prefix` followed by a `-` subtag boundary.
///
/// Compares on raw bytes rather than string slices so that untrusted,
/// non-ASCII input can never trigger a char-boundary panic. BCP-47 language
/// tags are ASCII, so a byte-level `eq_ignore_ascii_case` is exactly the
/// intended comparison; any multibyte range simply fails to match.
fn starts_with_language_boundary(value: &str, prefix: &str) -> bool {
    let (value_bytes, prefix_bytes) = (value.as_bytes(), prefix.as_bytes());
    value_bytes.len() > prefix_bytes.len()
        // In-bounds by the length guard above; never a char-boundary slice.
        && value_bytes[prefix_bytes.len()] == b'-'
        && value_bytes[..prefix_bytes.len()].eq_ignore_ascii_case(prefix_bytes)
}

/// Higher is more specific: exact match beats subtag match, which beats `*`.
pub(crate) fn language_match_specificity(range: &str, language: &str) -> usize {
    if range.trim() == "*" {
        0
    } else if range.eq_ignore_ascii_case(language) {
        usize::MAX
    } else {
        range.trim().len().min(language.trim().len())
    }
}

/// The primary subtag of a language tag, e.g. `en` for `en-US`.
pub(crate) fn primary_subtag(language: &str) -> &str {
    language
        .split_once('-')
        .map(|(primary, _)| primary)
        .unwrap_or(language)
}

#[cfg(test)]
mod tests {
    use super::super::types::PreferredLanguage;
    use super::*;

    #[test]
    fn default_quality_is_one_thousand_millis() {
        let language = PreferredLanguage::parse("de");
        assert_eq!(language.ranges().len(), 1);
        assert_eq!(language.ranges()[0].quality_millis(), MAX_QUALITY_MILLIS);
        assert_eq!(language.quality_millis_for("de"), 1000);
    }

    #[test]
    fn parses_three_decimal_q_values_precisely() {
        let language = PreferredLanguage::parse("en;q=0.333, fr;q=0.5, es;q=0.005");

        assert_eq!(language.quality_millis_for("en"), 333);
        assert_eq!(language.quality_millis_for("fr"), 500);
        assert_eq!(language.quality_millis_for("es"), 5);
    }

    #[test]
    fn valid_one_quality_forms_are_max_quality() {
        let language = PreferredLanguage::parse("en;q=1, fr;q=1.0, es;q=1.000");

        assert_eq!(language.quality_millis_for("en"), MAX_QUALITY_MILLIS);
        assert_eq!(language.quality_millis_for("fr"), MAX_QUALITY_MILLIS);
        assert_eq!(language.quality_millis_for("es"), MAX_QUALITY_MILLIS);
    }

    #[test]
    fn malformed_quality_is_rejected_as_zero() {
        let language =
            PreferredLanguage::parse("en;q=abc, fr;q=, es;q=0.x9, de;q=2.5, ja;q=1.001, ko;q=.5");

        assert_eq!(language.quality_millis_for("en"), 0);
        assert_eq!(language.quality_millis_for("fr"), 0);
        assert_eq!(language.quality_millis_for("es"), 0);
        assert_eq!(language.quality_millis_for("de"), 0);
        assert_eq!(language.quality_millis_for("ja"), 0);
        assert_eq!(language.quality_millis_for("ko"), 0);
        assert!(language.is_empty());
    }

    #[test]
    fn extra_fraction_digits_are_rejected() {
        let language = PreferredLanguage::parse("en;q=0.1239");
        assert_eq!(language.quality_millis_for("en"), 0);
    }

    #[test]
    fn whitespace_in_header_items_is_trimmed() {
        let language = PreferredLanguage::parse("  fr-CA ;  q=0.8 ,  en ; q=0.6  ");

        assert_eq!(language.ranges().len(), 2);
        assert_eq!(language.ranges()[0].tag(), "fr-CA");
        assert_eq!(language.ranges()[0].quality_millis(), 800);
        assert_eq!(language.ranges()[1].tag(), "en");
        assert_eq!(language.ranges()[1].quality_millis(), 600);
    }

    #[test]
    fn parse_quality_millis_handles_boundaries_directly() {
        assert_eq!(parse_quality_millis("0"), 0);
        assert_eq!(parse_quality_millis("0.0"), 0);
        assert_eq!(parse_quality_millis("0.5"), 500);
        assert_eq!(parse_quality_millis("1"), MAX_QUALITY_MILLIS);
        assert_eq!(parse_quality_millis(""), 0);
        assert_eq!(parse_quality_millis("1.5"), 0);
    }

    #[test]
    fn language_matches_is_bidirectional_and_wildcard_aware() {
        assert!(language_matches("*", "anything"));
        assert!(language_matches("en", "en-US"));
        assert!(language_matches("en-US", "en"));
        assert!(language_matches("EN", "en"));
        assert!(!language_matches("en", "fr"));
        // "en" must not match "english" (no subtag boundary).
        assert!(!language_matches("en", "english"));
    }

    #[test]
    fn language_matches_never_panics_on_multibyte_input() {
        // Regression: a range whose byte length exceeds the candidate but whose
        // boundary byte falls mid-char (e.g. "eñ" = 65 C3 B1) used to slice the
        // str at a non-char-boundary and panic. It must now simply not match.
        assert!(!language_matches("eñ", "en"));
        assert!(!language_matches("en", "eñ"));
        // Multibyte on the boundary byte itself and longer multibyte tails.
        assert!(!language_matches("en日本語", "en"));
        assert!(!language_matches("en", "en日本語"));
        assert!(!language_matches("café", "caf"));
        assert!(!language_matches("Ω", "en"));
        // A genuine ASCII subtag boundary still matches after the fix.
        assert!(language_matches("en-US", "en"));
    }

    #[test]
    fn starts_with_language_boundary_is_boundary_safe() {
        // Direct exercise of the helper across the tricky byte layouts.
        assert!(!starts_with_language_boundary("eñ", "en"));
        assert!(!starts_with_language_boundary("enñ", "en")); // boundary byte is C3
        assert!(starts_with_language_boundary("en-US", "en"));
        assert!(starts_with_language_boundary("EN-us", "en")); // case-insensitive
        assert!(!starts_with_language_boundary("english", "en")); // no boundary
        assert!(!starts_with_language_boundary("en", "en")); // equal length
    }

    #[test]
    fn specificity_orders_exact_over_subtag_over_wildcard() {
        assert_eq!(language_match_specificity("*", "en-US"), 0);
        assert_eq!(language_match_specificity("en-US", "en-US"), usize::MAX);
        assert!(
            language_match_specificity("en", "en-US")
                < language_match_specificity("en-US", "en-US")
        );
    }

    #[test]
    fn primary_subtag_strips_region() {
        assert_eq!(primary_subtag("en-US"), "en");
        assert_eq!(primary_subtag("zh"), "zh");
    }

    #[test]
    fn parse_accept_language_caps_range_count() {
        // F2 regression: a pathological header must be bounded so downstream
        // matching cannot be driven into quadratic blowup.
        let mut header = String::from("en");
        for n in 0..(MAX_LANGUAGE_RANGES * 8) {
            header.push_str(&format!(", en-{n};q=0.5"));
        }
        let ranges = parse_accept_language(&header);

        assert_eq!(ranges.len(), MAX_LANGUAGE_RANGES);
        // The earliest (highest-preference) ranges are the ones retained.
        assert_eq!(ranges[0].tag(), "en");
        assert_eq!(ranges[1].tag(), "en-0");
    }

    #[test]
    fn normalize_fallback_substitutes_default_for_empty() {
        assert_eq!(
            normalize_fallback("  ".to_string()),
            DEFAULT_FALLBACK_LANGUAGE
        );
        assert_eq!(normalize_fallback(" zh-CN ".to_string()), "zh-CN");
    }
}
