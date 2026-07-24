use core::iter::Peekable;

use proc_macro::{Delimiter, Ident, Span, TokenStream, TokenTree};

use crate::{
    helper::{
        expect_any_ident, expect_end, expect_group_consume_return_inner, expect_ident_consume,
        expect_punct_consume, generate_compile_error, into_peekable_iter, match_ident_consume,
    },
    outer_attr::{OuterAttr, parse_outer_attrs},
};

use super::next_anonymous_ident;

/// Common syntax captured from an endpoint or outpoint body declaration.
pub(crate) struct APHandlerDef {
    attrs: OuterAttr,
    is_pub: bool,
    name: Ident,
    protocol: Ident,
    request: Ident,
    body: TokenStream,
}

impl APHandlerDef {
    pub(crate) fn new(
        attrs: OuterAttr,
        is_pub: bool,
        name: Ident,
        protocol: Ident,
        request: Ident,
        body: TokenStream,
    ) -> Self {
        Self {
            attrs,
            is_pub,
            name,
            protocol,
            request,
            body,
        }
    }

    pub(crate) fn attrs(&self) -> &OuterAttr {
        &self.attrs
    }

    pub(crate) fn is_pub(&self) -> bool {
        self.is_pub
    }

    pub(crate) fn name(&self) -> &Ident {
        &self.name
    }

    pub(crate) fn protocol(&self) -> &Ident {
        &self.protocol
    }

    pub(crate) fn request(&self) -> &Ident {
        &self.request
    }

    pub(crate) fn body(&self) -> &TokenStream {
        &self.body
    }

    pub(crate) fn into_parts(self) -> (OuterAttr, bool, Ident, Ident, Ident, TokenStream) {
        (
            self.attrs,
            self.is_pub,
            self.name,
            self.protocol,
            self.request,
            self.body,
        )
    }

    /// Parses one complete trans-style handler declaration.
    pub(crate) fn from_trans_stream(
        tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<Self, TokenStream> {
        let attrs = parse_outer_attrs(tokens)?;
        let is_pub = match_ident_consume(tokens, "pub");
        let name = parse_route_name(tokens)?;
        let protocol = parse_protocol(tokens)?;
        let body = expect_group_consume_return_inner(
            tokens,
            Delimiter::Brace,
            "expected the handler body in `{...}`",
        )?;
        expect_end(
            tokens,
            "unexpected token after the trans handler declaration",
        )?;

        Ok(Self::new(
            attrs,
            is_pub,
            name,
            protocol,
            Ident::new("req", Span::call_site()),
            body,
        ))
    }

    /// Parses one complete function item, including its outer attributes.
    pub(crate) fn from_fn_item_stream(
        tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<Self, TokenStream> {
        let attrs = parse_outer_attrs(tokens)?;
        Self::from_fn_stream(tokens, attrs)
    }

    /// Parses a function item after its outer attributes were collected.
    pub(crate) fn from_fn_stream(
        tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
        attrs: OuterAttr,
    ) -> Result<Self, TokenStream> {
        let is_pub = match_ident_consume(tokens, "pub");
        expect_ident_consume(tokens, "fn", "expected `fn`")?;
        let name = parse_route_name(tokens)?;
        let protocol = parse_protocol(tokens)?;
        let parameters = expect_group_consume_return_inner(
            tokens,
            Delimiter::Parenthesis,
            "expected function parameters in `(...)`",
        )?;
        let request = parse_request_name(parameters)?;
        let body = expect_group_consume_return_inner(
            tokens,
            Delimiter::Brace,
            "expected the handler body in `{...}`",
        )?;
        expect_end(tokens, "unexpected token after the function declaration")?;

        Ok(Self::new(attrs, is_pub, name, protocol, request, body))
    }
}

fn parse_route_name(
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<Ident, TokenStream> {
    if match_ident_consume(tokens, "_") {
        Ok(next_anonymous_ident())
    } else {
        expect_any_ident(tokens, "expected a route name or anonymous marker `_`")
    }
}

fn parse_protocol(
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<Ident, TokenStream> {
    expect_punct_consume(tokens, "<", "expected `<` before the protocol")?;
    let protocol = expect_any_ident(tokens, "expected a protocol identifier")?;
    expect_punct_consume(tokens, ">", "expected `>` after the protocol")?;
    Ok(protocol)
}

fn parse_request_name(parameters: TokenStream) -> Result<Ident, TokenStream> {
    let mut tokens = into_peekable_iter(parameters);
    let request = match tokens.next() {
        Some(TokenTree::Ident(request)) => request,
        Some(token) => {
            return Err(generate_compile_error(
                token.span(),
                "expected the request parameter name",
            ));
        }
        None => return Ok(Ident::new("req", Span::call_site())),
    };
    expect_end(
        &mut tokens,
        "unexpected token after the request parameter name",
    )?;
    Ok(request)
}
