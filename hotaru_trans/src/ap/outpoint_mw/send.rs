//! Rewrites the outpoint-only `send;` marker into one inner-chain call.

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

/// Emit `<request> = next(<request>).await?;`.
fn send_expansion(request: &Ident) -> Vec<TokenTree> {
    let arguments = TokenStream::from(TokenTree::Ident(request.clone()));

    vec![
        TokenTree::Ident(request.clone()),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Ident(Ident::new("next", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, arguments)),
        TokenTree::Punct(Punct::new('.', Spacing::Alone)),
        TokenTree::Ident(Ident::new("await", Span::call_site())),
        TokenTree::Punct(Punct::new('?', Spacing::Alone)),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]
}

/// Recursively rewrite `send;`, leaving other uses of `send` untouched.
pub(super) fn rewrite_send(input: TokenStream, request: &Ident) -> TokenStream {
    let mut output = TokenStream::new();
    let mut tokens = input.into_iter().peekable();

    while let Some(token) = tokens.next() {
        match &token {
            TokenTree::Ident(ident) if ident.to_string() == "send" => {
                if matches!(
                    tokens.peek(),
                    Some(TokenTree::Punct(punct)) if punct.as_char() == ';'
                ) {
                    tokens.next();
                    output.extend(send_expansion(request));
                } else {
                    output.extend(core::iter::once(token));
                }
            }
            TokenTree::Group(group) => {
                let rewritten = rewrite_send(group.stream(), request);
                let mut rewritten_group = Group::new(group.delimiter(), rewritten);
                rewritten_group.set_span(group.span());
                output.extend(core::iter::once(TokenTree::Group(rewritten_group)));
            }
            _ => output.extend(core::iter::once(token)),
        }
    }

    output
}
