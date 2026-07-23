use core::iter::Peekable;
use proc_macro::{Delimiter, Ident, Literal, Span, TokenStream, TokenTree};

use crate::helper::{
    ensure_string_literal, expect_any_ident, expect_end, expect_group_consume_return_inner,
    expect_string_literal_consume, generate_compile_error, into_peekable_iter,
};

/// How core should interpret the URL literal when the definition is bound.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum UrlMode {
    #[default]
    Pattern,
    Literal,
}

/// Syntax-level registration address.
///
/// `app` exists only on the macro side: it identifies the application whose
/// future registration hook will call `bind`.
pub(crate) struct RouteAddress {
    app: Ident,
    url: Literal,
    url_mode: UrlMode,
}

impl RouteAddress {
    pub(crate) fn new(app: Ident, url: Literal) -> Self {
        Self {
            app,
            url,
            url_mode: UrlMode::default(),
        }
    }

    pub(crate) fn with_url_mode(mut self, url_mode: UrlMode) -> Self {
        self.url_mode = url_mode;
        self
    }

    pub(crate) fn app(&self) -> &Ident {
        &self.app
    }

    pub(crate) fn url(&self) -> &Literal {
        &self.url
    }

    pub(crate) fn url_mode(&self) -> UrlMode {
        self.url_mode
    }

    /// Parses one complete route-address fragment.
    ///
    /// Accepted forms:
    /// - `APP.url("/x")`
    /// - `APP.lit_url("/x")`
    /// - `APP: "/x"`
    /// - `"/x"` (defaults to `APP.url`)
    pub(crate) fn from_stream(
        tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<Self, TokenStream> {
        let first = match tokens.next() {
            Some(token) => token,
            None => {
                return Err(generate_compile_error(
                    Span::call_site(),
                    "expected a route address",
                ));
            }
        };

        let address = match first {
            TokenTree::Literal(url) => {
                ensure_string_literal(&url)?;
                Self::new(Ident::new("APP", Span::call_site()), url)
            }
            TokenTree::Ident(app) => match tokens.next() {
                Some(TokenTree::Punct(punct)) if punct.as_char() == ':' => {
                    let url = expect_string_literal_consume(tokens)?;
                    Self::new(app, url)
                }
                Some(TokenTree::Punct(punct)) if punct.as_char() == '.' => {
                    let method = expect_any_ident(tokens, "expected `url` or `lit_url` after `.`")?;
                    let mode = match method.to_string().as_str() {
                        "url" => UrlMode::Pattern,
                        "lit_url" => UrlMode::Literal,
                        _ => {
                            return Err(generate_compile_error(
                                method.span(),
                                "expected `url` or `lit_url` after `.`",
                            ));
                        }
                    };
                    let inner = expect_group_consume_return_inner(
                        tokens,
                        Delimiter::Parenthesis,
                        "expected `(...)` after the route selector",
                    )?;
                    let mut inner = into_peekable_iter(inner);
                    let url = expect_string_literal_consume(&mut inner)?;
                    expect_end(
                        &mut inner,
                        "unexpected token after the route string literal",
                    )?;
                    Self::new(app, url).with_url_mode(mode)
                }
                Some(token) => {
                    return Err(generate_compile_error(
                        token.span(),
                        "expected `:` or `.` after the app identifier",
                    ));
                }
                None => {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        "expected `:` or `.` after the app identifier",
                    ));
                }
            },
            token => {
                return Err(generate_compile_error(
                    token.span(),
                    "expected an app identifier or a string literal",
                ));
            }
        };

        expect_end(tokens, "unexpected token after the route address")?;
        Ok(address)
    }
}
