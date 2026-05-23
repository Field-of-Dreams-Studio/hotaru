//! The `send;` marker — its expansion form and a TokenStream rewriter that
//! finds every occurrence in an outpoint body and substitutes the expansion.
//!
//! Within an `outpoint!` body, `send;` is shorthand for one inner-chain
//! invocation:
//!
//! ```ignore
//! send;
//! // expands to:
//! <req_var_name> = next(<req_var_name>).await?;
//! ```
//!
//! Where `<req_var_name>` is the context variable identifier from
//! `UrlFunc::req_var_name` (default `req`).
//!
//! The rewriter is intentionally conservative: it only fires on `Ident("send")`
//! when the very next token is `Punct(';')`. A bare `send` not followed by
//! `;` is left untouched — so code like `channel.send(msg)`, `let send = ...`,
//! and pattern bindings `Ok(send) => ...` all keep working inside outpoint
//! bodies.

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

/// Build the token sequence that `send;` expands to.
///
/// Emits: `<req_var_name> = next(<req_var_name>).await?;`
pub fn send_expansion(req_var_name: &Ident) -> Vec<TokenTree> {
    // next(<req_var_name>)
    let mut call_args = TokenStream::new();
    call_args.extend(std::iter::once(TokenTree::Ident(req_var_name.clone())));

    vec![
        // <req_var_name> = next(<req_var_name>).await?;
        TokenTree::Ident(req_var_name.clone()),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Ident(Ident::new("next", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, call_args)),
        TokenTree::Punct(Punct::new('.', Spacing::Alone)),
        TokenTree::Ident(Ident::new("await", Span::call_site())),
        TokenTree::Punct(Punct::new('?', Spacing::Alone)),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]
}

/// Walk a TokenStream and rewrite every occurrence of the `send;` marker
/// into the [`send_expansion`] token sequence.
///
/// Recurses into nested groups (braces, parens, brackets) so a `send;`
/// inside a nested scope — e.g. `if cond { send; }` — is also rewritten.
///
/// A bare `send` identifier not followed by `;` is emitted unchanged; this
/// keeps method calls (`x.send(y)`), local bindings (`let send = ...`), and
/// pattern bindings working inside outpoint bodies.
pub fn rewrite_send(input: TokenStream, req_var_name: &Ident) -> TokenStream {
    let mut output = TokenStream::new();
    let mut tokens = input.into_iter().peekable();

    while let Some(tok) = tokens.next() {
        match &tok {
            // Match `send` identifier followed by `;` — the marker.
            TokenTree::Ident(id) if id.to_string() == "send" => {
                let is_marker = matches!(
                    tokens.peek(),
                    Some(TokenTree::Punct(p)) if p.as_char() == ';'
                );
                if is_marker {
                    tokens.next(); // consume the `;`
                    output.extend(send_expansion(req_var_name));
                } else {
                    // Bare `send` — not the marker, emit unchanged.
                    output.extend(std::iter::once(tok));
                }
            }
            // Recurse into nested groups (braces, parens, brackets).
            TokenTree::Group(group) => {
                let new_inner = rewrite_send(group.stream(), req_var_name);
                output.extend(std::iter::once(TokenTree::Group(Group::new(
                    group.delimiter(),
                    new_inner,
                ))));
            }
            _ => {
                output.extend(std::iter::once(tok));
            }
        }
    }

    output
}

