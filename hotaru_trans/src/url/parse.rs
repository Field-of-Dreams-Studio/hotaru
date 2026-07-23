use core::iter::Peekable;

use proc_macro::{Delimiter, Ident, Span, TokenStream, TokenTree};

use crate::ap::next_anonymous_ident;
use crate::helper::*;
use crate::url::url_func::UrlFunc;
use crate::url::urlargs::UrlArgs;
use crate::url::urlexpr::UrlExpr;

/// Parse the attribute input into UrlAttr
/// endpoint/outpoint! {
///   <url-expr>,
///   middleware = [ ... ],  // Optional
///   config = [ ... ], // Optional
///   endpoint_name<Protocol> {
///     ...
///  }
/// }
/// Only enabled when trans feature is enabled
#[allow(dead_code)]
pub fn parse_trans(args: TokenStream) -> Result<UrlArgs, TokenStream> {
    /// Parse the function definition into UrlFunc
    fn parse_inner(
        tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<UrlFunc, TokenStream> {
        let attrs = parse_outer_attrs(tokens)?;
        let is_pub = match_ident_consume(tokens, "pub");
        let fn_name = match match_ident_consume(tokens, "_") {
            true => next_anonymous_ident(),
            false => expect_any_ident(
                tokens,
                "Expected function name, or anonymous function annotation '_'",
            )?,
        };
        let _ = expect_punct_consume(tokens, "<", "Expected '<' after function name")?;
        let protocol = expect_any_ident(tokens, "Expected protocol identifier after '<'")?;
        let _ = expect_punct_consume(tokens, ">", "Expected '>' after protocol identifier")?;
        let fn_cont = expect_group_consume_return_inner(
            tokens,
            Delimiter::Brace,
            "Expected function body inside braces",
        )?;

        Ok(UrlFunc::new(
            is_pub,
            fn_name,
            protocol,
            Ident::new("req", Span::call_site()),
            fn_cont,
            attrs,
        ))
    }

    let mut tokens = into_peekable_iter(args);
    let url_expr = expect_stream_before_comma_consume(
        &mut tokens,
        true,
        "Expected a comma after the operations",
    )?;

    let mut middlewares = None;
    let mut config = None;

    if match_ident_consume(&mut tokens, "middleware") {
        tokens.next(); // Consume the `=`
        middlewares = Some(expect_array_consume(
            &mut tokens,
            "Expected an array for middleware",
        )?);
    }

    match_punct_consume(&mut tokens, ","); // Optional separator for better readability between middleware and config

    if match_ident_consume(&mut tokens, "config") {
        tokens.next(); // Consume the `=`
        config = Some(expect_array_consume(
            &mut tokens,
            "Expected an array for config",
        )?);
    }

    match_punct_consume(&mut tokens, ","); // Optional separator for better readability between middleware and config

    return Ok(UrlArgs::new(
        UrlExpr::from_tokens(url_expr)?,
        config,
        middlewares,
        parse_inner(&mut tokens)?,
    ));
}

/// Expect to be in the following format:
/// #[endpoint]
/// #[url(...)] // Required, Refer to UrlExpr struct
/// #[config([ ... ])] // Optional
/// #[middleware([ ... ])] // Optional
/// pub fn endpoint_name<Protocol>() {
///    ...
/// }
/// Only enabled when semi_trans feature is enabled
#[allow(dead_code)]
pub fn parse_semi_trans(args: TokenStream) -> Result<UrlArgs, TokenStream> {
    let mut tokens = into_peekable_iter(args);

    let mut outer_attrs = parse_outer_attrs(&mut tokens)?;
    let url_expr_raw = outer_attrs.remove("url").ok_or(generate_compile_error(
        Span::call_site(),
        "Missing required 'url' attribute",
    ))?;
    let url_expr = OuterAttr::get_inners(url_expr_raw, "Expected url(...)")?;
    let config = outer_attrs
        .remove("config")
        .map(|ts| {
            expect_array_consume(
                &mut into_peekable_iter(OuterAttr::get_inners(ts, "Expected config([...])")?),
                "Expected an array for config",
            )
        })
        .unwrap_or(Ok(vec![]))?;
    let middleware = outer_attrs
        .remove("middleware")
        .map(|ts| {
            expect_array_consume(
                &mut into_peekable_iter(OuterAttr::get_inners(ts, "Expected middleware([...])")?),
                "Expected an array for middleware",
            )
        })
        .unwrap_or(Ok(vec![]))?;

    let is_pub = match_ident_consume(&mut tokens, "pub");
    let _ = expect_ident_consume(
        &mut tokens,
        "fn",
        "Expected 'fn' keyword for function definition",
    )?;
    let fn_name = match match_ident_consume(&mut tokens, "_") {
        true => next_anonymous_ident(),
        false => expect_any_ident(
            &mut tokens,
            "Expected function name, or anonymous function annotation '_'",
        )?,
    };
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
    let fn_cont = expect_group_consume_return_inner(
        &mut tokens,
        Delimiter::Brace,
        "Expected function body inside braces",
    )?;

    return Ok(UrlArgs::new(
        UrlExpr::from_tokens(url_expr)?,
        Some(config),
        Some(middleware),
        UrlFunc::new(
            is_pub,
            fn_name,
            protocol,
            req_var_name,
            fn_cont,
            outer_attrs,
        ),
    ));
}

/// Expect to be in the following format:
/// #[endpoint(UrlExpr, config = [...], middleware = [...])]
/// pub fn endpoint_name<Protocol>() {
///    ...
/// }
/// Only enabled when attr feature is enabled
#[allow(dead_code)]
pub fn parse_attr(attr: TokenStream, args: TokenStream) -> Result<UrlArgs, TokenStream> {
    let mut attr = into_peekable_iter(attr);
    let mut tokens = into_peekable_iter(args);

    // Parse attribute arguments
    let url_expr = expect_stream_before_comma_consume(&mut attr, false, "Expected URL Pattern")?;
    let mut middlewares = None;
    let mut config = None;
    if match_ident_consume(&mut attr, "middleware") {
        attr.next(); // Consume the `=`
        middlewares = Some(expect_array_consume(
            &mut attr,
            "Expected an array for middleware",
        )?);
    }
    if match_ident_consume(&mut attr, "config") {
        attr.next(); // Consume the `=`
        config = Some(expect_array_consume(
            &mut attr,
            "Expected an array for config",
        )?);
    }

    let outer_attrs = parse_outer_attrs(&mut tokens)?;
    let is_pub = match_ident_consume(&mut tokens, "pub");
    let _ = expect_ident_consume(
        &mut tokens,
        "fn",
        "Expected 'fn' keyword for function definition",
    )?;
    let fn_name = match match_ident_consume(&mut tokens, "_") {
        true => next_anonymous_ident(),
        false => expect_any_ident(
            &mut tokens,
            "Expected function name, or anonymous function annotation '_'",
        )?,
    };
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
    let fn_cont = expect_group_consume_return_inner(
        &mut tokens,
        Delimiter::Brace,
        "Expected function body inside braces",
    )?;

    return Ok(UrlArgs::new(
        UrlExpr::from_tokens(url_expr)?,
        config,
        middlewares,
        UrlFunc::new(
            is_pub,
            fn_name,
            protocol,
            req_var_name,
            fn_cont,
            outer_attrs,
        ),
    ));
}
