#[derive(Clone, Debug)] 
pub enum PathPattern { 
    Literal(String), // A literal path, e.g. "foo"
    Regex(String), // A regex path, e.g. "\d+" 
    Any, // A wildcard path, e.g. "*" 
    AnyPath, // A wildcard path with a trailing slash, e.g. "**" 
} 

impl PathPattern{
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

