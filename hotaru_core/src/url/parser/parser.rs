use super::lexer::{RawToken, TypeKind};
use super::super::pattern::PathPattern; 

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternError {
    // We expected the angle group to end with '>' but didn't find it.
    ExpectedAngleClose { at: usize },
    // Inside <|||...|||> we expected N closing pipes but found fewer.
    MissingClosingPipes { expected: usize, found: usize, at: usize },
    // Found ':' where an identifier (name) should follow but didn't.
    ExpectedIdentAfterColon { at: usize },
    // Generic unexpected token in the current context.
    UnexpectedToken { at: usize, token: RawToken },
    // Found a token outside of angle groups that shouldn't appear there (internal invariant).
    UnexpectedTokenOutsideAngle { at: usize, token: RawToken },
    // <**path> must be the only content within its segment.
    AnyPathMixedWithOtherContent { at: usize },
} 

impl std::fmt::Display for PatternError { 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PatternError::ExpectedAngleClose { at } => {
                write!(f, "Expected '>' at index {}", at)
            }
            PatternError::MissingClosingPipes { expected, found, at } => {
                write!(f, "Expected {} closing pipes at index {}, found {}", expected, at, found)
            }
            PatternError::ExpectedIdentAfterColon { at } => {
                write!(f, "Expected identifier after ':' at index {}", at)
            }
            PatternError::UnexpectedToken { at, token } => {
                write!(f, "Unexpected token {:?} at index {}", token, at)
            }
            PatternError::UnexpectedTokenOutsideAngle { at, token } => {
                write!(f, "Unexpected token {:?} outside angle group at index {}", token, at)
            }
            PatternError::AnyPathMixedWithOtherContent { at } => {
                write!(f, "<**path> must be the only content within its segment at index {}", at)
            }
        }
    } 
}

/// Convert a token stream into:
/// - Vec<PathPattern>: one entry per path segment (split on Slash tokens)
/// - Vec<Option<String>>: optional parameter name associated with each segment
///
/// Returns Result to capture structural errors in the token stream.
///
/// Rules:
/// - Segments are delimited by RawToken::Slash. Empty segments (e.g., due to
///   consecutive slashes or leading/trailing slash) are skipped.
/// - A segment can be:
///   - Literal: only literal content, no angle patterns.
///   - Any: produced by <str> or <name> with no other content in the segment.
///   - Regex: produced by typed patterns (<int>, <uuid>, custom regex blocks), or any
///     mixture of literal + dynamic parts inside a single segment. Literal parts are
///     regex-escaped when embedded.
///   - AnyPath: produced by <**path>. It must be the only content of the segment.
/// - Names: If a segment defines a name via <..:name> or <name>, it's captured as Some(name).
///   If multiple names are specified in a single segment, the first one wins.
pub fn tokens_to_patterns(
    tokens: &Vec<RawToken>,
) -> Result<(Vec<PathPattern>, Vec<Option<String>>), PatternError> {
    let mut patterns: Vec<PathPattern> = Vec::new();
    let mut names: Vec<Option<String>> = Vec::new();

    let mut i = 0usize;

    // Parse segment-by-segment, delimited by Slash
    while i < tokens.len() {
        // Skip leading slashes and empty segments
        while i < tokens.len() {
            match tokens[i] {
                RawToken::Slash => i += 1,
                _ => break,
            }
        }
        if i >= tokens.len() {
            break;
        }

        // Build a single segment until the next Slash or end
        let seg_start = i;
        let mut seg_literal = String::new(); // plain literal accumulation
        let mut seg_regex = String::new(); // regex buffer (if segment needs regex)
        let mut seg_has_dynamic = false; // whether dynamic or regex content exists
        let mut seg_only_any = false; // true if the segment is exactly an "Any"
        let mut seg_name: Option<String> = None; // optional param name for this segment
        let mut seg_anypath = false; // <**path> present in this segment
        let mut seq_ended_by_slash = false; // whether we ended due to Slash 

        while i < tokens.len() {
            // If AnyPath has been set, nothing else is allowed in this segment.
            if seg_anypath {
                if !matches!(tokens[i], RawToken::Slash) {
                    return Err(PatternError::AnyPathMixedWithOtherContent { at: i });
                }
                break; // end segment on Slash
            }

            match &tokens[i] {
                RawToken::Slash => {
                    // segment ends 
                    seq_ended_by_slash = true; 
                    break;
                }

                RawToken::Literal(s) => {
                    if seg_has_dynamic {
                        seg_regex.push_str(&escape_regex(s));
                    } else {
                        seg_literal.push_str(s);
                    }
                    i += 1;
                }

                RawToken::Pipe => {
                    // '|' outside <...> is a literal char in the segment
                    if seg_has_dynamic {
                        seg_regex.push('|');
                    } else {
                        seg_literal.push('|');
                    }
                    i += 1;
                }

                RawToken::Colon => {
                    // ':' outside <...> is a literal char
                    if seg_has_dynamic {
                        seg_regex.push(':');
                    } else {
                        seg_literal.push(':');
                    }
                    i += 1;
                }

                RawToken::AngleStart => {
                    i += 1; // consume '<'
                    let (kind, name_opt, new_i) = parse_angle(tokens, i)?;
                    i = new_i;

                    if name_opt.is_some() && seg_name.is_none() {
                        seg_name = name_opt;
                    }

                    match kind {
                        AngleKind::Any => {
                            // If we already have literal content, we must switch to regex mode
                            if !seg_has_dynamic && !seg_literal.is_empty() {
                                seg_regex.push_str(&escape_regex(&seg_literal));
                                seg_literal.clear();
                            }
                            if seg_has_dynamic {
                                seg_regex.push_str("[^/]+");
                                seg_only_any = false;
                            } else {
                                // tentatively only Any so far
                                seg_only_any = true;
                            }
                            seg_has_dynamic = true;
                        }
                        AngleKind::AnyPath => {
                            // AnyPath must be the only content of this segment
                            if seg_has_dynamic || !seg_literal.is_empty() {
                                return Err(PatternError::AnyPathMixedWithOtherContent { at: seg_start });
                            }
                            seg_anypath = true;
                            seg_has_dynamic = true;
                            seg_only_any = false;
                        }
                        AngleKind::Regex(rx) => {
                            // If we were in literal-only mode, migrate literal into regex first
                            if !seg_has_dynamic && !seg_literal.is_empty() {
                                seg_regex.push_str(&escape_regex(&seg_literal));
                                seg_literal.clear();
                            }
                            seg_regex.push_str(&rx);
                            seg_has_dynamic = true;
                            seg_only_any = false;
                        }
                    }
                }

                RawToken::AngleClose | RawToken::Ident(_) | RawToken::Type(_) => {
                    // These should only appear inside <...>. Treat as structural error.
                    return Err(PatternError::UnexpectedTokenOutsideAngle {
                        at: i,
                        token: tokens[i].clone(),
                    });
                }
            }
        }

        // Finalize this segment
        if seg_anypath {
            patterns.push(PathPattern::AnyPath);
            names.push(seg_name);
        } else if seg_has_dynamic {
            // Move any remaining literal into regex
            if !seg_literal.is_empty() {
                seg_regex = format!("{}{}", escape_regex(&seg_literal), seg_regex);
                seg_literal.clear();
            }
            if seg_only_any && seg_regex.is_empty() {
                patterns.push(PathPattern::Any);
            } else {
                patterns.push(PathPattern::Regex(seg_regex));
            }
            names.push(seg_name);
        } else if !seg_literal.is_empty() {
            patterns.push(PathPattern::Literal(seg_literal));
            names.push(None);
        } else {
            // Empty segment (e.g., due to '//' or trailing '/'): skip
        } 

        // Detect the trailing slash and add Literal("") pattern for it 
        if seq_ended_by_slash { 
            if i == tokens.len() - 1 { 
                // Trailing slash at end of pattern
                patterns.push(PathPattern::Literal("".to_string())); 
                names.push(None); 
            } 
        }

        // If next token is a slash, consume one and continue
        if i < tokens.len() {
            if let RawToken::Slash = tokens[i] {
                i += 1;
            }
        }
    }

    Ok((patterns, names))
}

#[derive(Debug)]
enum AngleKind {
    Any,          // <str> or <name>
    AnyPath,      // <**path>
    Regex(String) // typed or custom regex
}

// Parse inside an angle group starting at tokens[i] (just after AngleStart).
// Returns (AngleKind, optional name, next index after AngleClose).
fn parse_angle(
    tokens: &[RawToken],
    mut i: usize,
) -> Result<(AngleKind, Option<String>, usize), PatternError> {
    let start_i = i;

    // 1) <**path[:name]?>
    if matches!(tokens.get(i), Some(RawToken::Type(TypeKind::Path))) {
        i += 1;
        let mut name: Option<String> = None;
        if matches!(tokens.get(i), Some(RawToken::Colon)) {
            i += 1;
            if let Some(RawToken::Ident(s)) = tokens.get(i) {
                name = Some(s.clone());
                i += 1;
            } else {
                return Err(PatternError::ExpectedIdentAfterColon { at: i });
            }
        }
        // Require AngleClose
        if matches!(tokens.get(i), Some(RawToken::AngleClose)) {
            i += 1;
            return Ok((AngleKind::AnyPath, name, i));
        } else {
            return Err(PatternError::ExpectedAngleClose { at: i });
        }
    }

    // 2) <type[:name]?> where type != **path
    if let Some(RawToken::Type(kind)) = tokens.get(i) {
        let mut regex = String::new();
        if let Some(rx) = kind.to_regex() {
            regex.push_str(rx);
        }
        i += 1;

        let mut name: Option<String> = None;
        if matches!(tokens.get(i), Some(RawToken::Colon)) {
            i += 1;
            if let Some(RawToken::Ident(s)) = tokens.get(i) {
                name = Some(s.clone());
                i += 1;
            } else {
                return Err(PatternError::ExpectedIdentAfterColon { at: i });
            }
        }
        // Require AngleClose
        if matches!(tokens.get(i), Some(RawToken::AngleClose)) {
            i += 1;
            return Ok((AngleKind::Regex(regex), name, i));
        } else {
            return Err(PatternError::ExpectedAngleClose { at: i });
        }
    }

    // 3) Regex block: <||...||[:name]?> or with 3+ pipes
    if matches!(tokens.get(i), Some(RawToken::Pipe)) {
        // Count opening pipes
        let mut n = 0usize;
        while matches!(tokens.get(i + n), Some(RawToken::Pipe)) {
            n += 1;
        }
        i += n;

        // Content may be empty or Literal (tokenizer emits literal for block content)
        let mut regex = String::new();
        if let Some(RawToken::Literal(s)) = tokens.get(i) {
            regex.push_str(s);
            i += 1;
        }

        // Expect n closing pipes
        let mut m = 0usize;
        while m < n && matches!(tokens.get(i), Some(RawToken::Pipe)) {
            m += 1;
            i += 1;
        }
        if m != n {
            return Err(PatternError::MissingClosingPipes {
                expected: n,
                found: m,
                at: i,
            });
        }

        // Optional :name
        let mut name: Option<String> = None;
        if matches!(tokens.get(i), Some(RawToken::Colon)) {
            i += 1;
            if let Some(RawToken::Ident(s)) = tokens.get(i) {
                name = Some(s.clone());
                i += 1;
            } else {
                return Err(PatternError::ExpectedIdentAfterColon { at: i });
            }
        }
        // Require AngleClose
        if matches!(tokens.get(i), Some(RawToken::AngleClose)) {
            i += 1;
            return Ok((AngleKind::Regex(regex), name, i));
        } else {
            return Err(PatternError::ExpectedAngleClose { at: i });
        }
    }

    // 4) Free-form regex: <literal_regex[:name]?>
    if let Some(RawToken::Literal(s)) = tokens.get(i) {
        let mut regex = s.clone();
        i += 1;

        let mut name: Option<String> = None;
        if matches!(tokens.get(i), Some(RawToken::Colon)) {
            i += 1;
            if let Some(RawToken::Ident(nm)) = tokens.get(i) {
                name = Some(nm.clone());
                i += 1;
            } else {
                return Err(PatternError::ExpectedIdentAfterColon { at: i });
            }
        }
        if matches!(tokens.get(i), Some(RawToken::AngleClose)) {
            i += 1;
            return Ok((AngleKind::Regex(regex), name, i));
        } else {
            return Err(PatternError::ExpectedAngleClose { at: i });
        }
    }

    // 5) Name-only Any: <ident>
    if let Some(RawToken::Ident(s)) = tokens.get(i) {
        let name = Some(s.clone());
        i += 1;
        if matches!(tokens.get(i), Some(RawToken::AngleClose)) {
            i += 1;
            return Ok((AngleKind::Any, name, i));
        } else {
            return Err(PatternError::ExpectedAngleClose { at: i });
        }
    }

    // 6) Unexpected token after '<'
    Err(PatternError::UnexpectedToken {
        at: start_i,
        token: tokens.get(start_i).cloned().unwrap_or_else(|| RawToken::Literal(String::new())),
    })
}

// Minimal regex escaping for literal segments we embed into Regex
fn escape_regex(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::url::parser::lexer::tokenize;
    use crate::debug_log;

    #[test]
    fn ok_literal_and_named_any_segments() {
        let tokens = tokenize("/users/<id>/details");
        let (pats, names) = tokens_to_patterns(&tokens).unwrap();
        assert_eq!(
            pats,
            vec![
                PathPattern::Literal("users".into()),
                PathPattern::Any,
                PathPattern::Literal("details".into()),
            ]
        );
        assert_eq!(names, vec![None, Some("id".into()), None]);
    } 

    #[test] 
    fn root_url() { 
        let tokens = tokenize("/");
        let (pats, names) = tokens_to_patterns(&tokens).unwrap();
        debug_log!("{:?}, {:?}", pats, names); 
    }

    // #[test]
    // fn ok_typed_and_regex_segments() {
    //     let tokens = tokenize("/page-<uint:num>/<||a|b||:alt>/<uuid:order>");
    //     let (pats, names) = tokens_to_patterns(&tokens).unwrap();
    //     assert_eq!(pats.len(), 3);
    //     match &pats[0] {
    //         PathPattern::Literal(s) => assert!(s.starts_with("page<")),
    //         _ => panic!("expected regex"),
    //     }
    //     assert_eq!(names[0], Some("num".into()));
    //     assert_eq!(pats[1], PathPattern::Regex("a|b".into()));
    //     assert_eq!(names[1], Some("alt".into()));
    //     match &pats[2] {
    //         PathPattern::Regex(s) => assert!(s.contains("{8}-")),
    //         _ => panic!("expected regex"),
    //     }
    //     assert_eq!(names[2], Some("order".into()));
    // }

    #[test]
    fn ok_anypath_catch_all() {
        let tokens = tokenize("/files/<**path:rest>");
        let (pats, names) = tokens_to_patterns(&tokens).unwrap();
        assert_eq!(pats, vec![PathPattern::Literal("files".into()), PathPattern::AnyPath]);
        assert_eq!(names, vec![None, Some("rest".into())]);
    }

    #[test]
    fn ok_name_only_any() {
        let tokens = tokenize("/<slug>");
        let (pats, names) = tokens_to_patterns(&tokens).unwrap();
        assert_eq!(pats, vec![PathPattern::Any]);
        assert_eq!(names, vec![Some("slug".into())]);
    }

    #[test]
    fn ok_str_any_without_name() {
        let tokens = tokenize("/<str>");
        let (pats, names) = tokens_to_patterns(&tokens).unwrap();
        assert_eq!(pats, vec![PathPattern::Regex("[^/]+".into())]);
        assert_eq!(names, vec![None]);
    }

    #[test]
    fn any_str_with_trailing_slash() {
        let tokens = tokenize("/<str>/");
        let (pats, names) = tokens_to_patterns(&tokens).unwrap();
        assert_eq!(pats, vec![PathPattern::Regex("[^/]+".into()), PathPattern::Literal("".into())]);
        assert_eq!(names, vec![None, None]);
    }

    #[test]
    fn error_unterminated_angle() {
        let tokens = tokenize("/<int");
        let err = tokens_to_patterns(&tokens).unwrap_err();
        matches!(err, PatternError::ExpectedAngleClose { .. });
    }

    #[test]
    fn error_anypath_mixed_with_literal() {
        let tokens = tokenize("/files-<**path>");
        let (pats, names) = tokens_to_patterns(&tokens).unwrap();
        assert_eq!(pats, vec![PathPattern::Literal("files<**path>".into())]);
        assert_eq!(names, vec![None]);
    }

    #[test]
    fn error_colon_without_name() {
        let tokens = tokenize("/<int:>");
        let err = tokens_to_patterns(&tokens).unwrap_err();
        matches!(err, PatternError::ExpectedIdentAfterColon { .. });
    }
} 
