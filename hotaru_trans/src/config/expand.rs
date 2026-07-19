use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::helper::{generate_compile_error, use_core};

use super::{Cloneable, Config};

impl Config {
    /// Expand this syntax model into a `Params` or `ParamsClone` expression.
    pub(crate) fn expand(self) -> TokenStream {
        let (entries, cloneable) = self.into_parts();
        let container = match cloneable {
            Cloneable::Yes => "ParamsClone",
            Cloneable::No => "Params",
        };
        let params = Ident::new("__hotaru_params", Span::mixed_site());
        let mut body = TokenStream::new();

        // let mut __hotaru_params = ::...::extensions::<container>::default();
        body.extend([
            TokenTree::Ident(Ident::new("let", Span::call_site())),
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
            TokenTree::Ident(params.clone()),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        ]);
        body.extend(use_core(&["extensions", container, "default"]));
        body.extend([
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);

        for entry in entries {
            // __hotaru_params.set(entry);
            body.extend([
                TokenTree::Ident(params.clone()),
                TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                TokenTree::Ident(Ident::new("set", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, entry)),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            ]);
        }

        // The final expression returns the completed container.
        body.extend([TokenTree::Ident(params)]);
        TokenStream::from(TokenTree::Group(Group::new(Delimiter::Brace, body)))
    }
}

/// Parse and expand one complete standalone configuration-macro input.
pub(crate) fn parse_and_expand(input: TokenStream, cloneable: Cloneable) -> TokenStream {
    let mut stream = input.into_iter().peekable();
    let config = match Config::from_stream(&mut stream, cloneable) {
        Ok(config) => config,
        Err(error) => return error,
    };

    if let Some(token) = stream.next() {
        return generate_compile_error(
            token.span(),
            "Unexpected token after the configuration array",
        );
    }

    config.expand()
}
