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

/// Build the token sequence for the outpoint's final-handler closure.
///
/// The emitted closure is what sits at the end of the inner middleware chain
/// inside an outpoint's outer wrapper. It captures a cloneable channel
/// handle and the outpoint URL node from its enclosing scope, then on every
/// call invokes `P::send(&channel, &mut ctx, &outpoint)` and threads
/// `ProtocolFlow::Close` back into the channel's open state.
///
/// `protocol` is the protocol identifier (e.g. `HTTP`).
/// `req_var_name` is the context variable identifier (default `req`).
/// `channel_var` is the identifier of the channel variable in scope.
/// `outpoint_var` is the identifier of the outpoint URL-node variable in scope.
///
/// Generated form (with the four substitutions applied):
///
/// ```ignore
/// move |mut <req_var_name>| {
///     let <channel_var> = <channel_var>.clone();
///     let <outpoint_var> = <outpoint_var>.clone();
///     async move {
///         let flow = <<Protocol as ::hotaru::hotaru_core::protocol::Protocol>>::send(
///             &<channel_var>,
///             &mut <req_var_name>,
///             &<outpoint_var>,
///         ).await?;
///         if matches!(flow, ::hotaru::hotaru_core::protocol::ProtocolFlow::Close) {
///             <channel_var>.close();
///         }
///         Ok(<req_var_name>)
///     }
/// }
/// ```
pub fn outpoint_final_handler(
    protocol: &Ident,
    req_var_name: &Ident,
    channel_var: &Ident,
    outpoint_var: &Ident,
) -> TokenStream {
    // Inner async-move body.
    let mut async_body = TokenStream::new();

    // let flow = <<Protocol as ::hotaru::hotaru_core::protocol::Protocol>>::send(&<channel>, &mut <req>, &<outpoint>).await?;
    async_body.extend(vec![
        TokenTree::Ident(Ident::new("let", Span::call_site())),
        TokenTree::Ident(Ident::new("flow", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        // <Protocol as ::hotaru::hotaru_core::protocol::Protocol>::send
        TokenTree::Punct(Punct::new('<', Spacing::Alone)),
        TokenTree::Ident(protocol.clone()),
        TokenTree::Ident(Ident::new("as", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("hotaru", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("hotaru_core", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("protocol", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("Protocol", Span::call_site())),
        TokenTree::Punct(Punct::new('>', Spacing::Alone)),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("send", Span::call_site())),
        // (&<channel>, &mut <req>, &<outpoint>)
        TokenTree::Group(Group::new(Delimiter::Parenthesis, {
            let mut args = TokenStream::new();
            args.extend(vec![
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(channel_var.clone()),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(Ident::new("mut", Span::call_site())),
                TokenTree::Ident(req_var_name.clone()),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(outpoint_var.clone()),
            ]);
            args
        })),
        TokenTree::Punct(Punct::new('.', Spacing::Alone)),
        TokenTree::Ident(Ident::new("await", Span::call_site())),
        TokenTree::Punct(Punct::new('?', Spacing::Alone)),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);

    // if matches!(flow, ::hotaru::hotaru_core::protocol::ProtocolFlow::Close) { <channel>.close(); }
    async_body.extend(vec![
        TokenTree::Ident(Ident::new("if", Span::call_site())),
        TokenTree::Ident(Ident::new("matches", Span::call_site())),
        TokenTree::Punct(Punct::new('!', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, {
            let mut g = TokenStream::new();
            g.extend(vec![
                TokenTree::Ident(Ident::new("flow", Span::call_site())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                // ::hotaru::hotaru_core::protocol::ProtocolFlow::Close
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("hotaru", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("hotaru_core", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("protocol", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("ProtocolFlow", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Close", Span::call_site())),
            ]);
            g
        })),
        TokenTree::Group(Group::new(Delimiter::Brace, {
            let mut g = TokenStream::new();
            // <channel>.close();
            g.extend(vec![
                TokenTree::Ident(channel_var.clone()),
                TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                TokenTree::Ident(Ident::new("close", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            ]);
            g
        })),
    ]);

    // Ok(<req_var_name>)
    async_body.extend(vec![
        TokenTree::Ident(Ident::new("Ok", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, {
            let mut g = TokenStream::new();
            g.extend(std::iter::once(TokenTree::Ident(req_var_name.clone())));
            g
        })),
    ]);

    // Outer closure body: clone the captures, then the async move block.
    let mut closure_body = TokenStream::new();

    // let <channel> = <channel>.clone();
    closure_body.extend(vec![
        TokenTree::Ident(Ident::new("let", Span::call_site())),
        TokenTree::Ident(channel_var.clone()),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Ident(channel_var.clone()),
        TokenTree::Punct(Punct::new('.', Spacing::Alone)),
        TokenTree::Ident(Ident::new("clone", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);

    // let <outpoint> = <outpoint>.clone();
    closure_body.extend(vec![
        TokenTree::Ident(Ident::new("let", Span::call_site())),
        TokenTree::Ident(outpoint_var.clone()),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Ident(outpoint_var.clone()),
        TokenTree::Punct(Punct::new('.', Spacing::Alone)),
        TokenTree::Ident(Ident::new("clone", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);

    // async move { ... }
    closure_body.extend(vec![
        TokenTree::Ident(Ident::new("async", Span::call_site())),
        TokenTree::Ident(Ident::new("move", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Brace, async_body)),
    ]);

    // move |mut <req_var_name>| { ... }
    let mut tokens = TokenStream::new();
    tokens.extend(vec![
        TokenTree::Ident(Ident::new("move", Span::call_site())),
        TokenTree::Punct(Punct::new('|', Spacing::Alone)),
        TokenTree::Ident(Ident::new("mut", Span::call_site())),
        TokenTree::Ident(req_var_name.clone()),
        TokenTree::Punct(Punct::new('|', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Brace, closure_body)),
    ]);

    tokens
}
