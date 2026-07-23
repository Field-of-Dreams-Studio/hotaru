mod chain;
mod func;

use core::iter::Peekable;

use proc_macro::{Delimiter, Ident, Span, TokenStream, TokenTree};

use crate::helper::{
    expect_any_ident, expect_end, expect_group_consume_return_inner, expect_ident_consume,
    expect_punct_consume, into_peekable_iter, match_ident_consume,
};
use crate::outer_attr::parse_outer_attrs;

pub(crate) use chain::{MWChain, MWSlot};
pub(crate) use func::MWFunc;

pub(crate) fn parse_trans(input: TokenStream) -> Result<MWFunc, TokenStream> {
    let mut tokens = into_peekable_iter(input);
    let attrs = parse_outer_attrs(&mut tokens)?;
    let is_pub = match_ident_consume(&mut tokens, "pub");
    let name = expect_any_ident(&mut tokens, "Expect middleware name")?;
    let _ = expect_punct_consume(&mut tokens, "<", "Expect '<' before the protocol type")?;
    let protocol = expect_any_ident(&mut tokens, "Protocol Identifier is Expected")?;
    let _ = expect_punct_consume(&mut tokens, ">", "Expect '>' after the protocol type")?;
    let cont = expect_group_consume_return_inner(
        &mut tokens,
        Delimiter::Brace,
        "Expect '{' for middleware content",
    )?;
    Ok(MWFunc::new(
        is_pub,
        name,
        protocol,
        Ident::new("req", Span::call_site()),
        cont,
        attrs,
    ))
}

/// Parse middleware declaration by using proc_macro_attribute-like syntax
/// e.g.
/// #[middleware]
/// pub fn my_middleware(req: Protocol) {
///    // middleware body
/// }
pub(crate) fn parse_semi_trans_or_attr(input: TokenStream) -> Result<MWFunc, TokenStream> {
    let mut tokens = into_peekable_iter(input);
    let attrs = parse_outer_attrs(&mut tokens)?;
    let is_pub = match_ident_consume(&mut tokens, "pub");
    let _ = expect_ident_consume(&mut tokens, "fn", "Expect 'fn' for middleware function")?;
    let name = expect_any_ident(&mut tokens, "Expect middleware function name")?;
    let _ = expect_punct_consume(&mut tokens, "<", "Expected '<' after function name")?;
    let protocol = expect_any_ident(&mut tokens, "Expected protocol identifier after '<'")?;
    let _ = expect_punct_consume(&mut tokens, ">", "Expected '>' after protocol identifier")?;
    let mut group = into_peekable_iter(expect_group_consume_return_inner(
        &mut tokens,
        Delimiter::Parenthesis,
        "Expected function parameters inside parentheses",
    )?);
    let req_var_name = match expect_any_ident(
        &mut group,
        "Expected request variable name as first parameter",
    ) {
        Ok(id) => id,
        Err(_) => Ident::new("req", Span::call_site()),
    };
    let cont = expect_group_consume_return_inner(
        &mut tokens,
        Delimiter::Brace,
        "Expect '{' for middleware content",
    )?;
    Ok(MWFunc::new(
        is_pub,
        name,
        protocol,
        req_var_name,
        cont,
        attrs,
    ))
}

pub(crate) fn parse_mw_chain(
    stream: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<TokenStream, TokenStream> {
    let chain = MWChain::from_stream(stream)?;
    expect_end(stream, "unexpected token after the middleware chain")?;
    Ok(chain.expand_middleware_chain())
}
