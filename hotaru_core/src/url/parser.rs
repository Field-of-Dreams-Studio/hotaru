use crate::url::{parser::parser::PatternError, PathPattern};
use std::collections::HashMap;

pub(self) mod lexer; 
pub(self) mod parser; 

// # `parse` – Compile a URL pattern string into matchable path patterns and capture names

// Signature:
// ```rust
// pub fn parse<T: AsRef<str>>(input: T) -> Result<(Vec<PathPattern>, Vec<String>), String> {
//     parser::tokens_to_patterns(&lexer::tokenize(input.as_ref()))
// }
// ```

// Purpose:
// - Turn a human-friendly route pattern string into:
//   - Vec<PathPattern>: a sequence of per-segment matchers the router can use when building a matcher.
//   - Vec<String>: the ordered list of parameter names for captured segments (only segments that declare names).

// Overview of the pipeline:
// - Lex: The input string is first tokenized by lexer::tokenize into RawToken values, handling escaping, angle groups, and separators.
// - Parse: The token stream is then converted by parser::tokens_to_patterns into the semantic representation of path segments and optionally named captures.
// - Surface API: parse wraps both steps and converts structural parser errors into a String.

// Return values:
// - Ok((patterns, names)):
//   - patterns: one PathPattern per path segment (split on "/" via Slash tokens).
//     - PathPattern::Literal(String): matches an exact path segment (no "/").
//     - PathPattern::Any: matches exactly one non-slash segment (wildcard for a single segment).
//     - PathPattern::Regex(String): a regex snippet to match this segment; literal parts are escaped by the parser.
//     - PathPattern::AnyPath: a greedy multi-segment matcher (catch-all) that spans remaining path. This does not expand to a regex here.
//   - names: the ordered list of parameter names declared in the pattern, left-to-right, for those segments that have a name. Unnamed dynamic segments (e.g., <int>) generates a None in this list.

// - Err(String): a human-readable error describing structural issues in the pattern (derived from PatternError), such as:
//   - Unclosed angle group (‘< …’ without ‘>’)
//   - Missing closing pipes in a pipe-delimited regex block
//   - Colon without a following identifier in an angle group
//   - Misplaced tokens outside/inside angle groups
//   - Mixing a catch-all <**path> with other content in the same segment

// Input grammar (summary):
// - Segments are separated by "/".
// - Angle groups introduce dynamic parts:
//   - Types: <int[:name]?>, <uint[:name]?>, <decimal[:name]?>, <str[:name]?>, <uuid[:name]?>.
//     - These map to regex snippets. Example: int -> -?\d+, str -> [^/]+.
//     - Name is optional. If present, it is included in the names list.
//   - Name-only wildcard: <name> (no type) means “Any” for that segment and captures under that name.
//   - Catch-all: <**path[:name]?> produces AnyPath. It must be the only content of its segment. Optional name, if present, is included in names.
//   - Free-form regex inside an angle group:
//     - Pipe-delimited: <||...||[:name]?> (or with 3+ pipes) allows “...” to contain unescaped “|”.
//     - Free-form literal regex: <...[ :name]?> (until ":" or ">").
//     - If a name is provided (:name), the segment becomes a named capture.
// - Escaping angle brackets in the outer literal:
//   - The lexer treats "-<" as a literal "<", and "->" as a literal ">". The dash is not included in output.

// How segments are produced:
// - A segment is everything between two "/" separators.
// - Literal characters outside angle groups append to a Literal segment. If a segment mixes literals with dynamic content, the entire segment becomes PathPattern::Regex, and literal parts are escaped before concatenation with dynamic regex parts.
// - PathPattern::Any is used when a segment is exactly a single wildcard (e.g., <str>, <name>) and nothing else.
// - PathPattern::AnyPath is used when the segment is exactly a catch-all (e.g., <**path[:name]?>). It cannot be combined with other text in the same segment.
// - Empty segments (leading “/”, trailing “/”, or “//”) are skipped.

// Names list semantics:
// - names contains only declared parameter names from angle groups with “:name” or from name-only groups like <slug>.
// - Unnamed dynamic segments (e.g., <int>) do not add an entry to names.
// - The order of names matches the left-to-right order in the pattern and corresponds to the order of capture groups you’ll generate when building the actual matching regex.

// Examples:
// - Literal and a named wildcard
//   - Input: "/users/<id>/details"
//   - Output:
//     - patterns: [Literal("users"), Any, Literal("details")]
//     - names: [None, Some("id"), None]
// - Typed and regex segments
//   - Input: "/page-<uint:page>/<||a|b||:alt>/<uuid:order>"
//   - Output (simplified):
//     - patterns: [Regex("page-\\d+"), Regex("a|b"), Regex("(?i)[0-9a-f]{8}-...") ]
//     - names: ["page", "alt", "order"]
// - Catch-all
//   - Input: "/files/<**path:rest>"
//   - Output:
//     - patterns: [Literal("files"), AnyPath]
//     - names: ["rest"]
// - Unnamed dynamic segments
//   - Input: "/<str>/<int>"
//   - Output:
//     - patterns: [Any, Regex("-?\\d+")]
//     - names: []  // no declared names, captures are positional only (if you choose to expose them)
// - Escaping angles
//   - Input: "foo-<bar->baz"
//   - Output:
//     - patterns: [Literal("foo<bar>baz")]
//     - names: []

// Error cases:
// - "/<int" -> Err("ExpectedAngleClose at index …")
// - "/<int:>" -> Err("ExpectedIdentAfterColon at index …")
// - "/files-<**path>" -> Err("AnyPathMixedWithOtherContent at index …")

// Complexity:
// - Tokenization and parsing both run in O(n) over the input size; memory scales with the number of segments and dynamic groups.

// Notes:
// - Type-to-regex mapping is handled by TypeKind::to_regex (e.g., int, uint, decimal, str, uuid). The special Path kind (for <**path>) does not expand to a regex here.
// - The parse function intentionally returns names as a compact list of declared names only, not a parallel vector to patterns. Use the order to align with your generated capture groups during regex building. 
pub fn parse<T: AsRef<str>>(input: T) -> Result<(Vec<PathPattern>, Vec<Option<String>>), PatternError> {
    parser::tokens_to_patterns(&lexer::tokenize(input.as_ref())) 
} 

/// Substitute parameters into a parsed URL pattern to generate a final URL path.
pub fn substitute(
    patterns: &[PathPattern],
    names: &[Option<String>],
    params: &HashMap<String, String>,
) -> Result<String, String> {
    if patterns.len() != names.len() {
        return Err("Pattern and name list length mismatch".to_string());
    }

    let mut segments: Vec<String> = Vec::with_capacity(patterns.len());

    for (idx, pattern) in patterns.iter().enumerate() {
        match pattern {
            PathPattern::Literal(lit) => {
                segments.push(lit.clone());
            }
            PathPattern::Any | PathPattern::Regex(_) => {
                let name = names
                    .get(idx)
                    .and_then(|opt| opt.as_ref())
                    .ok_or_else(|| format!("Missing parameter name at index {}", idx))?;
                let value = params
                    .get(name)
                    .ok_or_else(|| format!("Missing parameter: {}", name))?;
                segments.push(value.clone());
            }
            PathPattern::AnyPath => {
                let name = names
                    .get(idx)
                    .and_then(|opt| opt.as_ref())
                    .ok_or_else(|| format!("Missing parameter name at index {}", idx))?;
                let value = params
                    .get(name)
                    .ok_or_else(|| format!("Missing parameter: {}", name))?;
                segments.push(value.trim_start_matches('/').to_string());
                break;
            }
        }
    }

    let path = segments.join("/");
    if path.is_empty() {
        Ok("/".to_string())
    } else {
        Ok(format!("/{}", path))
    }
}

#[cfg(test)]
mod test {
    use super::{parse, substitute};
    use std::collections::HashMap;

    #[test]
    fn test_url_substitution() {
        let (patterns, names) = parse("/users/<id>/posts/<post_id>").unwrap();
        let mut params = HashMap::new();
        params.insert("id".to_string(), "123".to_string());
        params.insert("post_id".to_string(), "456".to_string());

        assert_eq!(
            substitute(&patterns, &names, &params).unwrap(),
            "/users/123/posts/456"
        );
    }

    #[test]
    fn test_url_substitution_missing_param() {
        let (patterns, names) = parse("/users/<id>").unwrap();
        let params: HashMap<String, String> = HashMap::new();
        assert!(substitute(&patterns, &names, &params).is_err());
    }

    #[test]
    fn test_url_substitution_typed_params() {
        let (patterns, names) = parse("/page-<uint:page>").unwrap();
        let mut params = HashMap::new();
        params.insert("page".to_string(), "page-42".to_string());
        assert_eq!(
            substitute(&patterns, &names, &params).unwrap(),
            "/page-42"
        );
    }
}
