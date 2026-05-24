#[derive(Clone, Debug)]
pub enum PathPattern {
    Literal(String), // A literal path, e.g. "foo"
    Regex(String),   // A regex path, e.g. "\d+"
    Any,             // A wildcard path, e.g. "*"
    AnyPath,         // A wildcard path with a trailing slash, e.g. "**"
}

impl PathPattern {
    pub fn literal_path<T: Into<String>>(path: T) -> Self {
        Self::Literal(path.into())
    }

    pub fn regex_path<T: Into<String>>(path: T) -> Self {
        Self::Regex(path.into())
    }

    pub fn any() -> Self {
        Self::Any
    }

    pub fn any_path() -> Self {
        Self::AnyPath
    }

    pub fn is_any_path(&self) -> bool {
        matches!(self, PathPattern::AnyPath)
    }

    /// Check if this pattern matches the given segment
    pub fn matches(&self, segment: &str) -> bool {
        match self {
            PathPattern::Literal(literal) => literal == segment,
            PathPattern::Regex(regex_str) => {
                // Note: In production, consider caching the compiled regex
                regex::Regex::new(regex_str)
                    .map(|re| re.is_match(segment))
                    .unwrap_or(false)
            }
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

pub mod path_pattern_creator {
    use super::PathPattern;

    /// Creates a literal path pattern.
    /// This is a wrapper around the literal_path function.
    /// This is useful for creating path patterns that are not regex.
    pub fn literal_path<T: Into<String>>(path: T) -> PathPattern {
        PathPattern::Literal(path.into())
    }

    pub fn trailing_slash() -> PathPattern {
        PathPattern::Literal("".to_string())
    }

    /// Creates a regex path pattern.
    /// This is a wrapper around the regex_path function.
    /// This is useful for creating path patterns that are regex.
    pub fn regex_path<T: Into<String>>(path: T) -> PathPattern {
        PathPattern::Regex(path.into())
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
            (PathPattern::Regex(l), PathPattern::Regex(r)) => l == r,
            (PathPattern::Any, PathPattern::Any) => true,
            (PathPattern::AnyPath, PathPattern::AnyPath) => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for PathPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathPattern::Literal(path) => write!(f, "Literal: {}", path),
            PathPattern::Regex(path) => write!(f, "Regex: {}", path),
            PathPattern::Any => write!(f, "*"),
            PathPattern::AnyPath => write!(f, "**"),
        }
    }
}

#[cfg(test)]
mod tests {
    //! `PartialEq` tests for `PathPattern`.
    //! 
    //! - Literal patterns: string equality
    //! - Wildcards (`Any` / `AnyPath`): kind equality (different kinds are unequal)
    //! - Regex patterns: identical source-string equality
    //! - Different variants are never equal

    use super::PathPattern;

    // ------------------------------------------------------------------
    // Reflexivity — every variant equals itself
    // ------------------------------------------------------------------

    #[test]
    fn literal_is_reflexive() {
        let p = PathPattern::literal_path("users");
        assert_eq!(p, p.clone());
    }

    #[test]
    fn regex_is_reflexive() {
        let p = PathPattern::regex_path(r"\d+");
        assert_eq!(p, p.clone());
    }

    #[test]
    fn any_is_reflexive() {
        assert_eq!(PathPattern::Any, PathPattern::Any);
    }

    #[test]
    fn any_path_is_reflexive() {
        assert_eq!(PathPattern::AnyPath, PathPattern::AnyPath);
    }

    // ------------------------------------------------------------------
    // Literal — string equality
    // ------------------------------------------------------------------

    #[test]
    fn literal_same_string_is_equal() {
        assert_eq!(
            PathPattern::literal_path("users"),
            PathPattern::literal_path("users"),
        );
    }

    #[test]
    fn literal_different_string_is_not_equal() {
        assert_ne!(
            PathPattern::literal_path("users"),
            PathPattern::literal_path("posts"),
        );
    }

    #[test]
    fn literal_case_sensitive() {
        // Path patterns are case-sensitive — `Users` and `users` are distinct.
        assert_ne!(
            PathPattern::literal_path("Users"),
            PathPattern::literal_path("users"),
        );
    }

    #[test]
    fn literal_empty_string_is_equal_to_itself() {
        // The empty-string literal is the root-endpoint case — important for
        // `UrlRegistration::Root` / `register_lit_named("", ...)` workflows.
        assert_eq!(
            PathPattern::literal_path(""),
            PathPattern::literal_path(""),
        );
    }

    #[test]
    fn literal_empty_string_is_not_any_wildcard() {
        // Despite the visual similarity, an empty literal is not the same as
        // a wildcard. This is important for the root-endpoint contract.
        assert_ne!(PathPattern::literal_path(""), PathPattern::Any);
        assert_ne!(PathPattern::literal_path(""), PathPattern::AnyPath);
    }

    // ------------------------------------------------------------------
    // Regex — identical source-string equality
    // ------------------------------------------------------------------

    #[test]
    fn regex_same_source_is_equal() {
        assert_eq!(
            PathPattern::regex_path(r"\d+"),
            PathPattern::regex_path(r"\d+"),
        );
    }

    #[test]
    fn regex_different_source_is_not_equal() {
        assert_ne!(
            PathPattern::regex_path(r"\d+"),
            PathPattern::regex_path(r"\w+"),
        );
    }

    #[test]
    fn regex_equivalent_but_different_source_is_not_equal() {
        // Equality is by source string, NOT by language. `[0-9]+` and `\d+`
        // match the same set but are textually distinct.
        assert_ne!(
            PathPattern::regex_path(r"\d+"),
            PathPattern::regex_path(r"[0-9]+"),
        );
    }

    // ------------------------------------------------------------------
    // Wildcards — kind equality
    // ------------------------------------------------------------------

    #[test]
    fn any_does_not_equal_any_path() {
        // `Any` matches one segment; `AnyPath` matches multiple. Different
        // kinds, different priorities (2 vs 3), must not compare equal.
        assert_ne!(PathPattern::Any, PathPattern::AnyPath);
    }

    // ------------------------------------------------------------------
    // Cross-variant inequality
    // ------------------------------------------------------------------

    #[test]
    fn literal_and_regex_are_not_equal_even_if_strings_match() {
        // `Literal("\\d+")` and `Regex("\\d+")` share a source string but
        // are different variants — they must not be equal.
        assert_ne!(
            PathPattern::literal_path(r"\d+"),
            PathPattern::regex_path(r"\d+"),
        );
    }

    #[test]
    fn literal_and_wildcards_are_not_equal() {
        let lit = PathPattern::literal_path("anything");
        assert_ne!(lit, PathPattern::Any);
        assert_ne!(lit, PathPattern::AnyPath);
    }

    #[test]
    fn regex_and_wildcards_are_not_equal() {
        let re = PathPattern::regex_path(r".*");
        assert_ne!(re, PathPattern::Any);
        assert_ne!(re, PathPattern::AnyPath);
    }

    // ------------------------------------------------------------------
    // Symmetry — a == b ⟹ b == a
    // ------------------------------------------------------------------

    #[test]
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
    fn path_slice_equality_matches_pattern_equality() {
        // `AccessPoints::refresh_path` compares `&[PathPattern]` slices via
        // the derived slice `==`, which delegates element-wise to
        // `PathPattern::eq`. This test pins that delegation.
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
}
