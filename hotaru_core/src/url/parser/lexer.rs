#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawToken {
    // Plain URL literals (outside <...> or free-form content inside).
    Literal(String),
    // Names like <name> or after ":" (e.g., <int:id> -> id).
    Ident(String),
    // Strongly-typed kinds recognized at the beginning of <...> before ":".
    // Path is special and does not expand to a regex.
    Type(TypeKind),
    // Single pipe token; multiple consecutive pipes will appear as repeated Pipe tokens.
    Pipe,
    // ":" separator between pattern and name.
    Colon,
    // "/" path separator (only emitted outside <...>)
    Slash,
    // "<"
    AngleStart,
    // ">"
    AngleClose,
}

// Decision: keep type/regex before name (<int:id>, <||re||:id>).
// This already supports:
// - <int> => Type(Int) without a name
// - <id>  => Ident("id") without a type
// - <**path> => Type(Path) special (no regex expansion)

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    // Signed integer, e.g., -123
    Int,
    // Unsigned integer, e.g., 123
    UInt,
    // Decimal number, e.g., -12.34
    Decimal,
    // Non-slash segment (a single path segment)
    Str,
    // Canonical UUID (case-insensitive, 8-4-4-4-12 hex)
    Uuid,
    // Special multi-segment catch-all. Does NOT expand to regex here.
    Path,
}

impl TypeKind {
    // Try to recognize a type from an identifier (before the colon).
    // Keep this minimal per design: only int, uint, decimal, str, uuid.
    pub fn from_ident(ident: &str) -> Option<Self> {
        match ident {
            "int" => Some(Self::Int),
            "uint" => Some(Self::UInt),
            "decimal" => Some(Self::Decimal),
            "str" => Some(Self::Str),
            "uuid" => Some(Self::Uuid),
            _ => None,
        }
    }

    // Convert to a regex snippet if applicable. Path is special and returns None.
    // These are intended for composing a full route regex later.
    pub fn to_regex(&self) -> Option<&'static str> {
        match self {
            TypeKind::Int => Some(r"-?\d+"),
            TypeKind::UInt => Some(r"\d+"),
            TypeKind::Decimal => Some(r"-?\d+(?:\.\d+)?"),
            TypeKind::Str => Some(r"[^/]+"),
            TypeKind::Uuid => Some(r"(?i)[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"),
            TypeKind::Path => None, // special: handled outside regex-building (e.g., greedy capture)
        }
    }
}

impl RawToken {
    fn push_literal(buf: &mut String, out: &mut Vec<RawToken>) {
        if !buf.is_empty() {
            out.push(RawToken::Literal(std::mem::take(buf)));
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

// Counts consecutive '|' characters starting at index i.
fn count_pipes(chars: &[char], mut i: usize) -> usize {
    let mut n = 0;
    while i < chars.len() && chars[i] == '|' {
        n += 1;
        i += 1;
    }
    n
}

// Tokenize the URL pattern string into RawToken sequence.
//
// Notes on behavior:
// - We treat "\" followed by "<" (i.e., "\<") as a literal "<" (escape for '<').
//   Similarly, "\>" is treated as literal ">" (escape for '>').
//   The '\' used for escaping is NOT included in output.
// - Outside of <...>:
//   - "/" is emitted as RawToken::Slash (segment separator).
//   - All other characters accumulate into Literal.
// - Inside <...>:
//   - If the very first thing is N pipes (N>=1), we enter a "regex-block" delimited
//     by exactly N consecutive pipes on each side: <|||content with | inside|||:name>.
//     Inside the block, single pipes do not become Pipe tokens; they are part of the Literal.
//     We emit: AngleStart, N*Pipe, Literal(content), N*Pipe, Colon, Ident(name), AngleClose.
//   - Else if it starts with "**path", we emit Type(Path).
//   - Else if it starts with an identifier and we have not yet seen ":", we emit Type(..)
//     when the ident is one of {int, uint, decimal, str, uuid}; otherwise Ident(ident).
//   - Else, we treat content as free-form regex until ":" or ">" (pipes and '/' inside are literal).
//   - After ":", we always treat following identifiers as Ident (parameter names).
//
// This function only performs tokenization (string chopping + escape handling). It does not validate
// the grammar beyond simple classification of idents/types.
pub fn tokenize(input: &str) -> Vec<RawToken> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    let mut out = Vec::new();
    let mut lit_buf = String::new();

    // State
    let mut in_angle = false;
    let mut after_angle_start = false; // true immediately after pushing AngleStart
    let mut saw_colon_in_angle = false;

    // Regex-block delimiter count (when Some(n), we're inside the content of a regex block)
    let mut regex_block_delim: Option<usize> = None;
    // Free-form regex mode: we are collecting everything until ":" or ">".
    let mut freeform_regex_until_colon = false;

    while i < chars.len() {
        // Escape handling for angles: "\<" -> "<", "\>" -> ">"
        if i + 1 < chars.len() && chars[i] == '\\' {
            match chars[i + 1] {
                '<' => {
                    // literal '<'
                    lit_buf.push('<');
                    i += 2;
                    continue;
                }
                '>' => {
                    // literal '>'
                    lit_buf.push('>');
                    i += 2;
                    continue;
                }
                _ => {
                    // '\' not escaping angle; treat as literal
                    lit_buf.push('\\');
                    i += 1;
                    continue;
                }
            }
        }

        // If we're inside a regex block, consume until the closing N pipes.
        if let Some(n) = regex_block_delim {
            if chars[i] == '|' {
                let m = count_pipes(&chars, i);
                if m >= n {
                    // Close the regex block with exactly n pipes
                    RawToken::push_literal(&mut lit_buf, &mut out);
                    for _ in 0..n {
                        out.push(RawToken::Pipe);
                    }
                    i += n;
                    regex_block_delim = None;
                    // After a regex block, we expect ":" or ">" etc. Continue loop.
                    continue;
                } else {
                    // Fewer than n pipes; they are part of the regex literal content.
                    for _ in 0..m {
                        lit_buf.push('|');
                    }
                    i += m;
                    continue;
                }
            } else {
                // Regular char inside regex block content
                lit_buf.push(chars[i]);
                i += 1;
                continue;
            }
        }

        // If we're in free-form regex (not block-delimited), collect until ":" or ">".
        if freeform_regex_until_colon {
            match chars[i] {
                ':' => {
                    RawToken::push_literal(&mut lit_buf, &mut out);
                    out.push(RawToken::Colon);
                    saw_colon_in_angle = true;
                    freeform_regex_until_colon = false;
                    i += 1;
                    continue;
                }
                '>' => {
                    RawToken::push_literal(&mut lit_buf, &mut out);
                    out.push(RawToken::AngleClose);
                    in_angle = false;
                    after_angle_start = false;
                    saw_colon_in_angle = false;
                    i += 1;
                    continue;
                }
                _ => {
                    // Any other char (including '|' and '/') is literal part of the regex.
                    lit_buf.push(chars[i]);
                    i += 1;
                    continue;
                }
            }
        }

        let c = chars[i];

        if !in_angle {
            match c {
                '<' => {
                    // flush pending literal
                    RawToken::push_literal(&mut lit_buf, &mut out);
                    out.push(RawToken::AngleStart);
                    in_angle = true;
                    after_angle_start = true;
                    saw_colon_in_angle = false;
                    i += 1;
                }
                '/' => {
                    // segment separator
                    RawToken::push_literal(&mut lit_buf, &mut out);
                    out.push(RawToken::Slash);
                    i += 1;
                }
                '>' => {
                    // '>' outside angle is just literal (unless escaped which we handled above).
                    lit_buf.push('>');
                    i += 1;
                }
                _ => {
                    // plain literal outside angle
                    lit_buf.push(c);
                    i += 1;
                }
            }
            continue;
        }

        // Inside <...>
        if after_angle_start {
            // First thing after '<'
            if c == '|' {
                // Begin regex-block with N pipes
                let n = count_pipes(&chars, i);
                // Emit the opening N Pipe tokens
                for _ in 0..n {
                    out.push(RawToken::Pipe);
                }
                i += n;
                regex_block_delim = Some(n);
                after_angle_start = false;
                continue;
            }

            // "**path"
            if c == '*' {
                let remaining: String = chars[i..].iter().collect();
                if remaining.starts_with("**path") {
                    RawToken::push_literal(&mut lit_buf, &mut out); // should be empty
                    out.push(RawToken::Type(TypeKind::Path));
                    i += "**path".len();
                    after_angle_start = false;
                    continue;
                }
                // If it doesn't match **path, treat '*' as literal and fall through
            }

            // Ident or known type at start
            if is_ident_start(c) {
                let start = i;
                i += 1;
                while i < chars.len() && is_ident_continue(chars[i]) {
                    i += 1;
                }
                let ident: String = chars[start..i].iter().collect();
                if let Some(kind) = TypeKind::from_ident(&ident) {
                    out.push(RawToken::Type(kind));
                } else {
                    out.push(RawToken::Ident(ident));
                }
                after_angle_start = false;
                continue;
            }

            // Otherwise, treat as free-form regex until ":" or ">"
            freeform_regex_until_colon = true;
            // Don't consume current char here; the regex collection will handle it.
            continue;
        }

        // Not the very first thing after '<' anymore
        match c {
            ':' => {
                RawToken::push_literal(&mut lit_buf, &mut out);
                out.push(RawToken::Colon);
                saw_colon_in_angle = true;
                i += 1;
            }
            '>' => {
                RawToken::push_literal(&mut lit_buf, &mut out);
                out.push(RawToken::AngleClose);
                in_angle = false;
                after_angle_start = false;
                saw_colon_in_angle = false;
                i += 1;
            }
            '|' => {
                // If not in regex-block mode, a '|' here is just a Pipe token.
                RawToken::push_literal(&mut lit_buf, &mut out);
                out.push(RawToken::Pipe);
                i += 1;
            }
            _ => {
                // Ident after colon must be Ident, not Type.
                if is_ident_start(c) {
                    let start = i;
                    i += 1;
                    while i < chars.len() && is_ident_continue(chars[i]) {
                        i += 1;
                    }
                    let ident: String = chars[start..i].iter().collect();
                    if !saw_colon_in_angle {
                        if let Some(kind) = TypeKind::from_ident(&ident) {
                            out.push(RawToken::Type(kind));
                        } else {
                            out.push(RawToken::Ident(ident));
                        }
                    } else {
                        out.push(RawToken::Ident(ident));
                    }
                } else {
                    // Any other chars become part of a literal until a special char
                    lit_buf.push(c);
                    i += 1;
                }
            }
        }
    }

    // Flush trailing literal (outside of any angle)
    if !in_angle {
        RawToken::push_literal(&mut lit_buf, &mut out);
    } else {
        // Inside angle at EOF: flush whatever we have (as literal/ident already pushed where applicable)
        RawToken::push_literal(&mut lit_buf, &mut out);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::{tokenize, RawToken::*, TypeKind};

    #[test]
    fn example_double_pipes_regex() {
        // <||\d+||:name> =>
        // AngleStart, Pipe, Pipe, Literal(\d+), Pipe, Pipe, Colon, Ident(name), AngleClose
        let input = "<||\\d+||:name>";
        let tokens = tokenize(input);
        let expected = vec![
            AngleStart,
            Pipe, Pipe,
            Literal("\\d+".to_string()),
            Pipe, Pipe,
            Colon,
            super::RawToken::Ident("name".into()),
            AngleClose,
        ];
        assert_eq!(tokens, expected);
    }

    #[test] 
    fn root_url() { 
        let input = "/"; 
        let tokens = tokenize(input);
        let expected = vec![
            Slash
        ]; 
        assert_eq!(tokens, expected);
    }

    #[test]
    fn any_ident() {
        let input = "<id>";
        let tokens = tokenize(input);
        let expected = vec![
            AngleStart,
            super::RawToken::Ident("id".into()),
            AngleClose,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn type_without_name() {
        let input = "<int>";
        let tokens = tokenize(input);
        let expected = vec![
            AngleStart,
            super::RawToken::Type(TypeKind::Int),
            AngleClose,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn special_type_and_name_int() {
        let input = "<int:id>";
        let tokens = tokenize(input);
        let expected = vec![
            AngleStart,
            super::RawToken::Type(TypeKind::Int),
            Colon,
            super::RawToken::Ident("id".into()),
            AngleClose,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn special_type_and_name_uint() {
        let input = "<uint:id>";
        let tokens = tokenize(input);
        let expected = vec![
            AngleStart,
            super::RawToken::Type(TypeKind::UInt),
            Colon,
            super::RawToken::Ident("id".into()),
            AngleClose,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn special_type_decimal() {
        let input = "<decimal:price>";
        let tokens = tokenize(input);
        let expected = vec![
            AngleStart,
            super::RawToken::Type(TypeKind::Decimal),
            Colon,
            super::RawToken::Ident("price".into()),
            AngleClose,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn special_type_str() {
        let input = "<str:slug>";
        let tokens = tokenize(input);
        let expected = vec![
            AngleStart,
            super::RawToken::Type(TypeKind::Str),
            Colon,
            super::RawToken::Ident("slug".into()),
            AngleClose,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn uuid_type_with_name() {
        let input = "<uuid:order_id>";
        let tokens = tokenize(input);
        let expected = vec![
            AngleStart,
            super::RawToken::Type(TypeKind::Uuid),
            Colon,
            super::RawToken::Ident("order_id".into()),
            AngleClose,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn any_path() {
        let input = "<**path>";
        let tokens = tokenize(input);
        let expected = vec![
            AngleStart,
            super::RawToken::Type(TypeKind::Path),
            AngleClose,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn outside_literals_and_angle_with_slash_separators() {
        let input = "/users/<id>";
        let tokens = tokenize(input);
        let expected = vec![
            Slash,
            Literal("users".into()),
            Slash,
            AngleStart,
            super::RawToken::Ident("id".into()),
            AngleClose,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn trailing_slash() {
        let input = "/users/<id>/";
        let tokens = tokenize(input);
        let expected = vec![
            Slash,
            Literal("users".into()),
            Slash,
            AngleStart,
            super::RawToken::Ident("id".into()),
            AngleClose,
            Slash,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn root_only_slash_and_double_slash() {
        let tokens = tokenize("/");
        assert_eq!(tokens, vec![Slash]);

        let tokens = tokenize("//");
        assert_eq!(tokens, vec![Slash, Slash]);
    }

    #[test]
    fn escape_angles_with_backslash() {
        let input = "foo\\<bar\\>baz";
        let tokens = tokenize(input);
        let expected = vec![
            Literal("foo<bar>baz".into()),
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn triple_pipe_regex_with_internal_pipe() {
        let input = "<|||a|b|||:name>";
        let tokens = tokenize(input);
        let expected = vec![
            AngleStart,
            Pipe, Pipe, Pipe,
            Literal("a|b".into()),
            Pipe, Pipe, Pipe,
            Colon,
            super::RawToken::Ident("name".into()),
            AngleClose,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn freeform_regex_without_block_allows_single_pipes() {
        let input = "<a|b:c>";
        let tokens = tokenize(input);
        let expected = vec![
            AngleStart,
            Ident("a".into()),
            Pipe, 
            Ident("b".into()), 
            Colon,
            super::RawToken::Ident("c".into()),
            AngleClose,
        ];
        assert_eq!(tokens, expected);
    }

    #[test]
    fn typekind_to_regex_contract() {
        assert_eq!(TypeKind::Int.to_regex(), Some(r"-?\d+"));
        assert_eq!(TypeKind::UInt.to_regex(), Some(r"\d+"));
        assert_eq!(TypeKind::Decimal.to_regex(), Some(r"-?\d+(?:\.\d+)?"));
        assert_eq!(TypeKind::Str.to_regex(), Some(r"[^/]+"));
        assert_eq!(TypeKind::Uuid.to_regex(), Some(r"(?i)[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"));
        assert_eq!(TypeKind::Path.to_regex(), None);
    }
} 
