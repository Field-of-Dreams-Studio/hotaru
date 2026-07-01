#[cfg(not(feature = "std"))]
use crate::prelude::*;
use alloc::sync::Arc;

use crate::debug_warn;

// `regex` crate under all flavours. Under `full` the `regex/unicode`
// feature adds Unicode tables (for `\p{...}` classes etc.); under
// `lite`/`embedded` those are skipped for a smaller binary. Since
// `regex` 1.9 the crate compiles as genuine `no_std + alloc` with
// `default-features = false`, so no cfg-fork is needed.
use regex::Regex as CompiledRegex;

/// A regex segment paired with its compiled form.
///
/// Compilation happens once at construction (typically route registration).
/// Subsequent matches reuse the cached compiled regex.
///
/// Patterns are wrapped with `^(?:...)$` so a match consumes the entire
/// segment, matching the convention that one regex pattern matches one
/// URL segment (which never contains `/`).
///
/// Invalid patterns log a `debug_warn!` at construction and store
/// `re = None`; subsequent matches return `false`. This preserves the
/// previous "silently never matches" semantics while making the failure
/// observable in logs.
#[derive(Clone, Debug)]
pub struct RegexSegment {
    src: String,
    re: Option<Arc<CompiledRegex>>,
}

impl RegexSegment {
    /// Compiles `src` as a regex, anchored with `^(?:...)$`. Logs and
    /// stores `None` on compile failure.
    pub fn new<T: Into<String>>(src: T) -> Self {
        let src = src.into();
        let anchored = format!("^(?:{})$", src);
        let re = match CompiledRegex::new(&anchored) {
            Ok(re) => Some(Arc::new(re)),
            Err(_err) => {
                debug_warn!(
                    "PathPattern: failed to compile regex {:?}: {}",
                    src,
                    _err
                );
                None
            }
        };
        Self { src, re }
    }

    /// Returns the original (unanchored) source string the segment was
    /// constructed with.
    pub fn src(&self) -> &str {
        &self.src
    }

    /// Returns whether this segment matches `text` end-to-end.
    pub fn is_match(&self, text: &str) -> bool {
        match &self.re {
            Some(re) => re.is_match(text),
            None => false,
        }
    }

    /// Returns whether the pattern compiled successfully.
    pub fn is_compiled(&self) -> bool {
        self.re.is_some()
    }
}

/// A single URL path-segment matcher.
#[derive(Clone, Debug)]
pub enum PathPattern {
    /// A literal path segment, such as `users`.
    Literal(String),
    /// A regex path segment, compiled and anchored to one segment.
    Regex(RegexSegment),
    /// A wildcard matching exactly one path segment.
    Any,
    /// A catch-all wildcard matching the rest of the path.
    AnyPath,
}

impl PathPattern {
    /// Creates a literal path pattern.
    pub fn literal_path<T: Into<String>>(path: T) -> Self {
        Self::Literal(path.into())
    }

    /// Creates a regex path pattern.
    pub fn regex_path<T: Into<String>>(path: T) -> Self {
        Self::Regex(RegexSegment::new(path))
    }

    /// Creates a single-segment wildcard pattern.
    pub fn any() -> Self {
        Self::Any
    }

    /// Creates a catch-all wildcard pattern.
    pub fn any_path() -> Self {
        Self::AnyPath
    }

    /// Returns whether this pattern is a catch-all wildcard.
    pub fn is_any_path(&self) -> bool {
        matches!(self, PathPattern::AnyPath)
    }

    /// Check if this pattern matches the given segment.
    pub fn matches(&self, segment: &str) -> bool {
        match self {
            PathPattern::Literal(literal) => literal == segment,
            PathPattern::Regex(seg) => seg.is_match(segment),
            PathPattern::Any | PathPattern::AnyPath => true,
        }
    }

    /// Get the priority of this pattern for ordering (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            PathPattern::Literal(_) => 0,
            PathPattern::Regex(_) => 1,
            PathPattern::Any => 2,
            PathPattern::AnyPath => 3,
        }
    }
}

/// Convenience constructors for common path patterns.
pub mod path_pattern_creator {
    use alloc::string::{String, ToString};

    use super::{PathPattern, RegexSegment};

    /// Creates a literal path pattern.
    /// This is a wrapper around the literal_path function.
    /// This is useful for creating path patterns that are not regex.
    pub fn literal_path<T: Into<String>>(path: T) -> PathPattern {
        PathPattern::Literal(path.into())
    }

    /// Creates a literal empty segment used for trailing slash registration.
    pub fn trailing_slash() -> PathPattern {
        PathPattern::Literal("".to_string())
    }

    /// Creates a regex path pattern.
    /// This is a wrapper around the regex_path function.
    /// This is useful for creating path patterns that are regex.
    pub fn regex_path<T: Into<String>>(path: T) -> PathPattern {
        PathPattern::Regex(RegexSegment::new(path))
    }

    /// Creates a any pattern.
    /// You may use this to match any string.
    /// This is faster then regex when any string should be passed into the same endpoint
    pub fn any() -> PathPattern {
        PathPattern::Any
    }

    /// Creates a any path pattern.
    /// This is useful for matching any path.
    /// This is faster then regex when any path should be passed into the same endpoint
    pub fn any_path() -> PathPattern {
        PathPattern::AnyPath
    }
}

impl PartialEq for PathPattern {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PathPattern::Literal(l), PathPattern::Literal(r)) => l == r,
            (PathPattern::Regex(l), PathPattern::Regex(r)) => l.src == r.src,
            (PathPattern::Any, PathPattern::Any) => true,
            (PathPattern::AnyPath, PathPattern::AnyPath) => true,
            _ => false,
        }
    }
}

impl core::fmt::Display for PathPattern {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PathPattern::Literal(path) => write!(f, "Literal: {}", path),
            PathPattern::Regex(seg) => write!(f, "Regex: {}", seg.src),
            PathPattern::Any => write!(f, "*"),
            PathPattern::AnyPath => write!(f, "**"),
        }
    }
}

#[cfg(test)]
mod tests {
    //! `PartialEq` and matching tests for `PathPattern`.
    //!
    //! - Literal patterns: string equality
    //! - Wildcards (`Any` / `AnyPath`): kind equality (different kinds are unequal)
    //! - Regex patterns: identical source-string equality, anchored full-segment matching
    //! - Different variants are never equal

    use super::PathPattern;

    // ------------------------------------------------------------------
    // Reflexivity — every variant equals itself
    // ------------------------------------------------------------------

    #[test]
    /// A literal pattern equals a clone of itself.
    fn literal_is_reflexive() {
        let p = PathPattern::literal_path("users");
        assert_eq!(p, p.clone());
    }

    #[test]
    /// A regex pattern equals a clone of itself (source-string based).
    fn regex_is_reflexive() {
        let p = PathPattern::regex_path(r"\d+");
        assert_eq!(p, p.clone());
    }

    #[test]
    /// `Any` equals `Any` (variant kind equality).
    fn any_is_reflexive() {
        assert_eq!(PathPattern::Any, PathPattern::Any);
    }

    #[test]
    /// `AnyPath` equals `AnyPath` (variant kind equality).
    fn any_path_is_reflexive() {
        assert_eq!(PathPattern::AnyPath, PathPattern::AnyPath);
    }

    // ------------------------------------------------------------------
    // Literal — string equality
    // ------------------------------------------------------------------

    #[test]
    /// Two literals built from the same source compare equal.
    fn literal_same_string_is_equal() {
        assert_eq!(
            PathPattern::literal_path("users"),
            PathPattern::literal_path("users"),
        );
    }

    #[test]
    /// Two literals built from different sources compare unequal.
    fn literal_different_string_is_not_equal() {
        assert_ne!(
            PathPattern::literal_path("users"),
            PathPattern::literal_path("posts"),
        );
    }

    #[test]
    /// Path patterns are case-sensitive — `Users` and `users` are distinct.
    fn literal_case_sensitive() {
        assert_ne!(
            PathPattern::literal_path("Users"),
            PathPattern::literal_path("users"),
        );
    }

    #[test]
    /// The empty-string literal is the root-endpoint case — important for
    /// `UrlRegistration::Root` / `register_lit_named("", ...)` workflows.
    fn literal_empty_string_is_equal_to_itself() {
        assert_eq!(
            PathPattern::literal_path(""),
            PathPattern::literal_path(""),
        );
    }

    #[test]
    /// Despite the visual similarity, an empty literal is not the same as
    /// a wildcard. This is important for the root-endpoint contract.
    fn literal_empty_string_is_not_any_wildcard() {
        assert_ne!(PathPattern::literal_path(""), PathPattern::Any);
        assert_ne!(PathPattern::literal_path(""), PathPattern::AnyPath);
    }

    // ------------------------------------------------------------------
    // Regex — identical source-string equality
    // ------------------------------------------------------------------

    #[test]
    /// Two regex patterns built from the same source compare equal.
    fn regex_same_source_is_equal() {
        assert_eq!(
            PathPattern::regex_path(r"\d+"),
            PathPattern::regex_path(r"\d+"),
        );
    }

    #[test]
    /// Two regex patterns built from different sources compare unequal.
    fn regex_different_source_is_not_equal() {
        assert_ne!(
            PathPattern::regex_path(r"\d+"),
            PathPattern::regex_path(r"\w+"),
        );
    }

    #[test]
    /// Equality is by source string, NOT by language. `[0-9]+` and `\d+`
    /// match the same set but are textually distinct.
    fn regex_equivalent_but_different_source_is_not_equal() {
        assert_ne!(
            PathPattern::regex_path(r"\d+"),
            PathPattern::regex_path(r"[0-9]+"),
        );
    }

    // ------------------------------------------------------------------
    // Wildcards — kind equality
    // ------------------------------------------------------------------

    #[test]
    /// `Any` matches one segment; `AnyPath` matches multiple. Different
    /// kinds, different priorities (2 vs 3), must not compare equal.
    fn any_does_not_equal_any_path() {
        assert_ne!(PathPattern::Any, PathPattern::AnyPath);
    }

    // ------------------------------------------------------------------
    // Cross-variant inequality
    // ------------------------------------------------------------------

    #[test]
    /// `Literal("\\d+")` and `Regex("\\d+")` share a source string but
    /// are different variants — they must not be equal.
    fn literal_and_regex_are_not_equal_even_if_strings_match() {
        assert_ne!(
            PathPattern::literal_path(r"\d+"),
            PathPattern::regex_path(r"\d+"),
        );
    }

    #[test]
    /// A literal pattern is never equal to either wildcard variant.
    fn literal_and_wildcards_are_not_equal() {
        let lit = PathPattern::literal_path("anything");
        assert_ne!(lit, PathPattern::Any);
        assert_ne!(lit, PathPattern::AnyPath);
    }

    #[test]
    /// A regex pattern is never equal to either wildcard variant.
    fn regex_and_wildcards_are_not_equal() {
        let re = PathPattern::regex_path(r".*");
        assert_ne!(re, PathPattern::Any);
        assert_ne!(re, PathPattern::AnyPath);
    }

    // ------------------------------------------------------------------
    // Symmetry — a == b ⟹ b == a
    // ------------------------------------------------------------------

    #[test]
    /// `PartialEq` for `PathPattern` is symmetric across every variant.
    fn equality_is_symmetric() {
        let lit_a = PathPattern::literal_path("x");
        let lit_b = PathPattern::literal_path("x");
        assert_eq!(lit_a == lit_b, lit_b == lit_a);

        let re_a = PathPattern::regex_path("y");
        let re_b = PathPattern::regex_path("y");
        assert_eq!(re_a == re_b, re_b == re_a);

        assert_eq!(
            PathPattern::Any == PathPattern::AnyPath,
            PathPattern::AnyPath == PathPattern::Any,
        );
    }

    // ------------------------------------------------------------------
    // Slice equality — the shape `refresh_path` actually uses
    // ------------------------------------------------------------------

    #[test]
    /// `AccessPoints::refresh_path` compares `&[PathPattern]` slices via
    /// the derived slice `==`, which delegates element-wise to
    /// `PathPattern::eq`. This test pins that delegation.
    fn path_slice_equality_matches_pattern_equality() {
        let lhs = vec![
            PathPattern::literal_path(""),
            PathPattern::literal_path("users"),
            PathPattern::regex_path(r"\d+"),
        ];
        let rhs = vec![
            PathPattern::literal_path(""),
            PathPattern::literal_path("users"),
            PathPattern::regex_path(r"\d+"),
        ];
        assert_eq!(lhs.as_slice(), rhs.as_slice());

        // Differing in any element makes the slices unequal.
        let mut rhs_diff = rhs.clone();
        rhs_diff[1] = PathPattern::literal_path("posts");
        assert_ne!(lhs.as_slice(), rhs_diff.as_slice());

        // Differing only in the regex source string is enough.
        let mut rhs_re = rhs.clone();
        rhs_re[2] = PathPattern::regex_path(r"\w+");
        assert_ne!(lhs.as_slice(), rhs_re.as_slice());

        // Different length is unequal regardless of content.
        let shorter: Vec<PathPattern> = rhs.iter().take(2).cloned().collect();
        assert_ne!(lhs.as_slice(), shorter.as_slice());
    }

    // ------------------------------------------------------------------
    // Matching — anchored, end-to-end against a single segment
    // ------------------------------------------------------------------

    #[test]
    /// Literal patterns match only the exact segment, never a prefix or suffix.
    fn literal_matches_exact_segment() {
        assert!(PathPattern::literal_path("users").matches("users"));
        assert!(!PathPattern::literal_path("users").matches("user"));
        assert!(!PathPattern::literal_path("users").matches("users/extra"));
    }

    #[test]
    /// `[0-9]+` must NOT match `"123abc"` — the segment must consume the
    /// entire string. This is the anchoring fix. (Uses `[0-9]` rather than
    /// `\d` so the test is portable to the `lite`/`embedded` no_std flavour,
    /// which builds `regex` without unicode tables.)
    fn regex_matches_full_segment_only() {
        let p = PathPattern::regex_path(r"[0-9]+");
        assert!(p.matches("123"));
        assert!(!p.matches("123abc"));
        assert!(!p.matches("abc123"));
        assert!(!p.matches(""));
    }

    #[test]
    /// A pattern that already contains its own anchors must still work
    /// after the framework wraps it with `^(?:...)$`.
    fn regex_anchoring_handles_inner_anchors() {
        let p = PathPattern::regex_path(r"^user[0-9]+$");
        assert!(p.matches("user42"));
        assert!(!p.matches("user42x"));
    }

    #[test]
    /// An invalid regex logs a `debug_warn` and stores no compiled regex;
    /// the resulting pattern never matches anything.
    fn invalid_regex_constructs_but_never_matches() {
        let p = PathPattern::regex_path("[unclosed");
        assert!(!p.matches("anything"));
        assert!(!p.matches(""));
    }

    #[test]
    /// `Any` matches every single segment, including the empty one.
    fn any_matches_any_segment() {
        assert!(PathPattern::Any.matches("anything"));
        assert!(PathPattern::Any.matches(""));
    }

    #[test]
    /// `AnyPath` matches every segment (the catch-all multi-segment marker).
    fn any_path_matches_any_segment() {
        assert!(PathPattern::AnyPath.matches("anything"));
    }

    // ------------------------------------------------------------------
    // Built-in typed-route regexes — no_std / `lite` regression guard
    // ------------------------------------------------------------------

    /// Every built-in `TypeKind` regex must COMPILE (not degrade to
    /// `re = None`) and match correctly under EVERY feature flavour,
    /// including `lite`/`embedded` where `regex` is built without its
    /// unicode tables. Before the ASCII rewrite of `TypeKind::to_regex`,
    /// `\d` / `(?i)` failed to compile under `lite`, so `<int>` / `<uint>`
    /// / `<decimal>` / `<uuid>` routes silently never matched. This test
    /// fails loudly if that regression ever returns.
    #[test]
    fn typed_route_regexes_compile_and_match_under_every_flavour() {
        use super::RegexSegment;
        use crate::url::parser::TypeKind;

        // All non-Path kinds must compile.
        for kind in [
            TypeKind::Int,
            TypeKind::UInt,
            TypeKind::Decimal,
            TypeKind::Str,
            TypeKind::Uuid,
        ] {
            let src = kind.to_regex().expect("non-Path kinds expand to a regex");
            assert!(
                RegexSegment::new(src).is_compiled(),
                "typed-route regex for {:?} must compile under every flavour (src = {:?})",
                kind,
                src,
            );
        }
        // Path is intentionally not a regex.
        assert_eq!(TypeKind::Path.to_regex(), None);

        let seg = |k: TypeKind| RegexSegment::new(k.to_regex().unwrap());

        // Int — optional leading '-', ASCII digits, whole segment only.
        let int = seg(TypeKind::Int);
        assert!(int.is_match("123"));
        assert!(int.is_match("-42"));
        assert!(!int.is_match("12a"));
        assert!(!int.is_match(""));

        // UInt — no sign.
        let uint = seg(TypeKind::UInt);
        assert!(uint.is_match("7"));
        assert!(!uint.is_match("-7"));

        // Decimal — optional sign and fractional part; no bare/leading dot.
        let dec = seg(TypeKind::Decimal);
        assert!(dec.is_match("3"));
        assert!(dec.is_match("-3.14"));
        assert!(!dec.is_match("3."));
        assert!(!dec.is_match(".5"));

        // Uuid — 8-4-4-4-12 hex, case-insensitive via an explicit ASCII class.
        let uuid = seg(TypeKind::Uuid);
        assert!(uuid.is_match("550e8400-e29b-41d4-a716-446655440000"));
        assert!(uuid.is_match("550E8400-E29B-41D4-A716-446655440000"));
        assert!(!uuid.is_match("550e8400e29b41d4a716446655440000"));
        assert!(!uuid.is_match("zzze8400-e29b-41d4-a716-446655440000"));

        // ASCII-only is intentional: `[0-9]` never matches non-ASCII digits
        // (fullwidth "１２３") under any flavour, matching what the downstream
        // integer parser accepts.
        assert!(!uint.is_match("１２３"));

        // Str — any single non-slash segment.
        let s = seg(TypeKind::Str);
        assert!(s.is_match("hello"));
        assert!(!s.is_match("a/b"));
    }
}
