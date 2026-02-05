use hotaru_lib::random::random_alpha_string;
use std::iter::Peekable;

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::helper::*;
use crate::ctor::gen_ctor;

pub struct OutpointArgs {
    pub url_expr: OutpointExpr,
    pub config: Option<Vec<TokenStream>>,
    pub middlewares: Option<Vec<TokenStream>>,
    pub op: OutpointFunc,
}

impl OutpointArgs {
    pub fn new(
        url_expr: OutpointExpr,
        config: Option<Vec<TokenStream>>,
        middlewares: Option<Vec<TokenStream>>,
        op: OutpointFunc,
    ) -> Self {
        OutpointArgs {
            url_expr,
            config,
            middlewares,
            op,
        }
    }

    pub fn reg_func(&self) -> TokenStream {
        let ctor_attrs = gen_ctor();

        let mut reg_func = TokenStream::new();
        reg_func.extend(vec![TokenTree::Ident(Ident::new(
            &format!("__wrapper_{}", &self.op.fn_name),
            Span::call_site(),
        ))]);

        // std::sync::Arc::new(__wrapper_xxx)
        let mut arc_func = TokenStream::new();
        arc_func.extend(vec![
            TokenTree::Ident(Ident::new("std", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("sync", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("Arc", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("new", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, reg_func)),
        ]);

        let modified_url_expr = self.url_expr.expand(self.op.protocol.clone());

        let mut cont = TokenStream::new();
        cont.extend(vec![
            TokenTree::Ident(Ident::new("let", Span::call_site())),
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
            TokenTree::Ident(Ident::new("outpoint", Span::call_site())),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Brace, modified_url_expr)),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            TokenTree::Ident(Ident::new("outpoint", Span::call_site())),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("set_method", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, arc_func)),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);

        if let Some(configs) = self.config.clone() {
            for expr in configs {
                cont.extend(vec![
                    TokenTree::Ident(Ident::new("outpoint", Span::call_site())),
                    TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("set_params", Span::call_site())),
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, expr)),
                    TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                ])
            }
        }

        if let Some(mws) = self.middlewares.clone() {
            let mut mw_decl = TokenStream::new();
            mw_decl.extend(vec![
                TokenTree::Ident(Ident::new("let", Span::call_site())),
                TokenTree::Ident(Ident::new("mut", Span::call_site())),
                TokenTree::Ident(Ident::new("middlewares", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Vec", Span::call_site())),
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                TokenTree::Ident(Ident::new("std", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("sync", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Arc", Span::call_site())),
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                TokenTree::Ident(Ident::new("dyn", Span::call_site())),
                TokenTree::Ident(Ident::new("hotaru", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("hotaru_core", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("app", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("middleware", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("AsyncMiddleware", Span::call_site())),
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                TokenTree::Ident(self.op.protocol.clone()),
                TokenTree::Ident(Ident::new("as", Span::call_site())),
                TokenTree::Ident(Ident::new("Protocol", Span::call_site())),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Context", Span::call_site())),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Punct(Punct::new('+', Spacing::Alone)),
                TokenTree::Punct(Punct::new('\'', Spacing::Joint)),
                TokenTree::Ident(Ident::new("static", Span::call_site())),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                TokenTree::Ident(Ident::new("vec", Span::call_site())),
                TokenTree::Punct(Punct::new('!', Spacing::Alone)),
                TokenTree::Group(Group::new(Delimiter::Bracket, TokenStream::new())),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            ]);

            cont.extend(mw_decl);

            for expr in mws {
                let is_dots = {
                    let tokens: Vec<TokenTree> = expr.clone().into_iter().collect();
                    tokens.len() == 2
                        && matches!(tokens.get(0), Some(TokenTree::Punct(p)) if p.as_char() == '.')
                        && matches!(tokens.get(1), Some(TokenTree::Punct(p)) if p.as_char() == '.')
                };

                if is_dots {
                    let mut block_content = TokenStream::new();
                    block_content.extend(vec![
                        TokenTree::Ident(Ident::new("let", Span::call_site())),
                        TokenTree::Ident(Ident::new("client_middlewares", Span::call_site())),
                        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                        TokenTree::Ident(self.url_expr.client.clone()),
                        TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("get_client_middlewares", Span::call_site())),
                        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                        TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                        TokenTree::Ident(self.op.protocol.clone()),
                        TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
                        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                    ]);

                    block_content.extend(vec![
                        TokenTree::Ident(Ident::new("middlewares", Span::call_site())),
                        TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("extend", Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                            let mut g = TokenStream::new();
                            g.extend(vec![TokenTree::Ident(Ident::new(
                                "client_middlewares",
                                Span::call_site(),
                            ))]);
                            g
                        })),
                        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                    ]);

                    cont.extend(vec![TokenTree::Group(Group::new(Delimiter::Brace, block_content))]);
                } else {
                    let mut arc_new = TokenStream::new();
                    arc_new.extend(vec![
                        TokenTree::Ident(Ident::new("std", Span::call_site())),
                        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("sync", Span::call_site())),
                        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("Arc", Span::call_site())),
                        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("new", Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, expr)),
                    ]);

                    let mut push_call = TokenStream::new();
                    push_call.extend(vec![
                        TokenTree::Ident(Ident::new("middlewares", Span::call_site())),
                        TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("push", Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, arc_new)),
                        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                    ]);
                    cont.extend(push_call);
                }
            }

            cont.extend(vec![
                TokenTree::Ident(Ident::new("outpoint", Span::call_site())),
                TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                TokenTree::Ident(Ident::new("set_middlewares", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                    let mut g = TokenStream::new();
                    g.extend(vec![TokenTree::Ident(Ident::new("middlewares", Span::call_site()))]);
                    g
                })),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            ]);
        }

        cont.extend(vec![
            TokenTree::Ident(Ident::new("outpoint", Span::call_site())),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("register", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                let mut g = TokenStream::new();
                g.extend(vec![
                    TokenTree::Ident(Ident::new("stringify", Span::call_site())),
                    TokenTree::Punct(Punct::new('!', Spacing::Alone)),
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                        let mut inner = TokenStream::new();
                        inner.extend(vec![TokenTree::Ident(self.op.fn_name.clone())]);
                        inner
                    })),
                ]);
                g
            })),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);

        let mut reg_func = TokenStream::new();
        reg_func.extend(ctor_attrs);
        reg_func.extend(vec![
            TokenTree::Ident(Ident::new("fn", Span::call_site())),
            TokenTree::Ident(Ident::new(
                &format!("__register_{}", self.op.fn_name),
                Span::call_site(),
            )),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
            TokenTree::Group(Group::new(Delimiter::Brace, cont)),
        ]);

        reg_func
    }
}

pub struct OutpointFunc {
    pub is_pub: bool,
    pub fn_name: Ident,
    pub protocol: Ident,
    pub req_var_name: Ident,
    pub fn_cont: TokenStream,
    pub attrs: OuterAttr,
}

impl OutpointFunc {
    pub fn new(
        is_pub: bool,
        fn_name: Ident,
        protocol: Ident,
        req_var_name: Ident,
        fn_cont: TokenStream,
        attrs: OuterAttr,
    ) -> Self {
        Self {
            is_pub,
            fn_name,
            protocol,
            req_var_name,
            fn_cont,
            attrs,
        }
    }

    pub fn generate_function(&self) -> TokenStream {
        let mut arguments = TokenStream::new();
        arguments.extend(vec![
            TokenTree::Ident(self.req_var_name.clone()),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Punct(Punct::new('&', Spacing::Alone)),
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Ident(self.protocol.clone()),
            TokenTree::Ident(Ident::new("as", Span::call_site())),
            TokenTree::Ident(Ident::new("Protocol", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("Context", Span::call_site())),
        ]);

        let mut tokens = TokenStream::new();
        tokens.extend(self.attrs.reform());

        if self.is_pub {
            tokens.extend(vec![TokenTree::Ident(Ident::new("pub", Span::call_site()))]);
        }
        tokens.extend(vec![
            TokenTree::Ident(Ident::new("async", Span::call_site())),
            TokenTree::Ident(Ident::new("fn", Span::call_site())),
            TokenTree::Ident(self.fn_name.clone()),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, arguments)),
            TokenTree::Punct(Punct::new('-', Spacing::Joint)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Ident(self.protocol.clone()),
            TokenTree::Ident(Ident::new("as", Span::call_site())),
            TokenTree::Ident(Ident::new("Protocol", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("Context", Span::call_site())),
            TokenTree::Ident(Ident::new("as", Span::call_site())),
            TokenTree::Ident(Ident::new("RequestContext", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("Response", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Brace, self.fn_cont.clone())),
        ]);
        tokens
    }

    pub fn wrapper_function(&self) -> TokenStream {
        let mut arguments = TokenStream::new();
        arguments.extend(vec![
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
            TokenTree::Ident(self.req_var_name.clone()),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Ident(self.protocol.clone()),
            TokenTree::Ident(Ident::new("as", Span::call_site())),
            TokenTree::Ident(Ident::new("Protocol", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("Context", Span::call_site())),
        ]);

        let mut internal_args = TokenStream::new();
        internal_args.extend(vec![
            TokenTree::Punct(Punct::new('&', Spacing::Alone)),
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
            TokenTree::Ident(self.req_var_name.clone()),
        ]);

        let mut cont = TokenStream::new();
        cont.extend(vec![
            TokenTree::Ident(Ident::new("let", Span::call_site())),
            TokenTree::Ident(Ident::new("response", Span::call_site())),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
            TokenTree::Ident(self.fn_name.clone()),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, internal_args)),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("await", Span::call_site())),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            TokenTree::Ident(self.req_var_name.clone()),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("response", Span::call_site())),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
            TokenTree::Ident(Ident::new("response", Span::call_site())),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            TokenTree::Ident(self.req_var_name.clone()),
        ]);

        let mut tokens = TokenStream::new();
        tokens.extend(vec![
            TokenTree::Ident(Ident::new("async", Span::call_site())),
            TokenTree::Ident(Ident::new("fn", Span::call_site())),
            TokenTree::Ident(Ident::new(
                &format!("__wrapper_{}", &self.fn_name),
                Span::call_site(),
            )),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, arguments)),
            TokenTree::Punct(Punct::new('-', Spacing::Joint)),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Ident(self.protocol.clone()),
            TokenTree::Ident(Ident::new("as", Span::call_site())),
            TokenTree::Ident(Ident::new("Protocol", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("Context", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Brace, cont)),
        ]);
        tokens
    }
}

pub struct OutpointExpr {
    pub client: Ident,
    method: Ident,
    literal: proc_macro::Literal,
}

impl OutpointExpr {
    pub fn from_tokens(input: TokenStream) -> Result<Self, TokenStream> {
        let mut tokens = into_peekable_iter(input);
        match tokens.peek() {
            Some(TokenTree::Ident(client_ident)) => {
                let client = client_ident.clone();
                tokens.next();
                match tokens.peek() {
                    Some(TokenTree::Punct(punct)) if punct.as_char() == ':' => {
                        tokens.next();
                        match tokens.next() {
                            Some(TokenTree::Literal(lit)) => Ok(OutpointExpr {
                                client,
                                method: Ident::new("url", Span::call_site()),
                                literal: lit,
                            }),
                            _ => Err(generate_compile_error(
                                Span::call_site(),
                                "Expected a string literal after ':'",
                            )),
                        }
                    }
                    Some(TokenTree::Punct(punct)) if punct.as_char() == '.' => {
                        tokens.next();
                        match tokens.next() {
                            Some(TokenTree::Ident(method_ident))
                                if method_ident.to_string() == "url"
                                    || method_ident.to_string() == "query" =>
                            {
                                let method = method_ident.clone();
                                match tokens.next() {
                                    Some(TokenTree::Group(group))
                                        if group.delimiter() == Delimiter::Parenthesis =>
                                    {
                                        let mut inner_tokens = group.stream().into_iter();
                                        match inner_tokens.next() {
                                            Some(TokenTree::Literal(lit)) => Ok(OutpointExpr {
                                                client,
                                                method,
                                                literal: lit,
                                            }),
                                            _ => Err(generate_compile_error(
                                                Span::call_site(),
                                                "Expected a string literal inside the parentheses",
                                            )),
                                        }
                                    }
                                    _ => Err(generate_compile_error(
                                        Span::call_site(),
                                        "Expected parentheses after method identifier",
                                    )),
                                }
                            }
                            _ => Err(generate_compile_error(
                                Span::call_site(),
                                "Expected 'url' or 'query' method identifier after '.'",
                            )),
                        }
                    }
                    _ => Err(generate_compile_error(
                        Span::call_site(),
                        "Expected ':' or '.' after client identifier",
                    )),
                }
            }
            Some(TokenTree::Literal(lit)) => Ok(OutpointExpr {
                client: Ident::new("CLIENT", Span::call_site()),
                method: Ident::new("url", Span::call_site()),
                literal: lit.clone(),
            }),
            _ => Err(generate_compile_error(
                Span::call_site(),
                "Expected a client identifier or a string literal for URL",
            )),
        }
    }

    pub fn expand(&self, protocol: Ident) -> TokenStream {
        let mut tokens = TokenStream::new();
        tokens.extend(vec![
            TokenTree::Ident(self.client.clone()),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(self.method.clone()),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Ident(protocol),
            TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            TokenTree::Ident(Ident::new("_", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                let mut g = TokenStream::new();
                g.extend(vec![TokenTree::Literal(self.literal.clone())]);
                g
            })),
        ]);
        tokens
    }
}

pub fn parse_trans(args: TokenStream) -> Result<OutpointArgs, TokenStream> {
    fn parse_inner(
        tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<OutpointFunc, TokenStream> {
        let attrs = parse_outer_attrs(tokens)?;
        let is_pub = match_ident_consume(tokens, "pub");
        let fn_name = match match_punct_consume(tokens, "_") {
            true => {
                let random_name = format!("auto_generated_{}", random_alpha_string(8));
                Ident::new(&random_name, Span::call_site())
            }
            false => expect_any_ident(tokens, "Expected function name")?,
        };
        let _ = expect_punct_consume(tokens, "<", "Expected '<' after function name")?;
        let protocol = expect_any_ident(tokens, "Expected protocol identifier after '<'")?;
        let _ = expect_punct_consume(tokens, ">", "Expected '>' after protocol identifier")?;
        let fn_cont =
            expect_group_consume_return_inner(tokens, Delimiter::Brace, "Expected function body")?;

        Ok(OutpointFunc::new(
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
        tokens.next();
        middlewares = Some(expect_array_consume(
            &mut tokens,
            "Expected an array for middleware",
        )?);
    }

    if match_ident_consume(&mut tokens, "config") {
        tokens.next();
        config = Some(expect_array_consume(
            &mut tokens,
            "Expected an array for config",
        )?);
    }

    Ok(OutpointArgs::new(
        OutpointExpr::from_tokens(url_expr)?,
        config,
        middlewares,
        parse_inner(&mut tokens)?,
    ))
}

pub fn parse_semi_trans(args: TokenStream) -> Result<OutpointArgs, TokenStream> {
    let mut tokens = into_peekable_iter(args);

    let mut outer_attrs = parse_outer_attrs(&mut tokens)?;
    let url_expr_raw = outer_attrs
        .remove("url")
        .ok_or(generate_compile_error(
            Span::call_site(),
            "Missing required 'url' attribute",
        ))?;
    let url_expr = OuterAttr::get_inners(url_expr_raw, "Expected url(...)")?;
    let config = outer_attrs
        .remove("config")
        .map(|ts| {
            expect_array_consume(
                &mut into_peekable_iter(OuterAttr::get_inners(
                    ts,
                    "Expected config([...])",
                )?),
                "Expected an array for config",
            )
        })
        .unwrap_or(Ok(vec![]))?;
    let middleware = outer_attrs
        .remove("middleware")
        .map(|ts| {
            expect_array_consume(
                &mut into_peekable_iter(OuterAttr::get_inners(
                    ts,
                    "Expected middleware([...])",
                )?),
                "Expected an array for middleware",
            )
        })
        .unwrap_or(Ok(vec![]))?;

    let is_pub = match_ident_consume(&mut tokens, "pub");
    let _ = expect_ident_consume(&mut tokens, "fn", "Expected 'fn' keyword")?;
    let fn_name = match match_punct_consume(&mut tokens, "_") {
        true => {
            let random_name = format!("auto_generated_{}", random_alpha_string(8));
            Ident::new(&random_name, Span::call_site())
        }
        false => expect_any_ident(&mut tokens, "Expected function name")?,
    };
    let _ = expect_punct_consume(&mut tokens, "<", "Expected '<' after function name")?;
    let protocol = expect_any_ident(&mut tokens, "Expected protocol identifier after '<'")?;
    let _ = expect_punct_consume(&mut tokens, ">", "Expected '>' after protocol identifier")?;
    let mut group = into_peekable_iter(expect_group_consume_return_inner(
        &mut tokens,
        Delimiter::Parenthesis,
        "Expected function parameters inside parentheses",
    )?);
    let req_var_name = match expect_any_ident(&mut group, "Expected request variable name") {
        Ok(id) => id,
        Err(_) => Ident::new("req", Span::call_site()),
    };
    let fn_cont =
        expect_group_consume_return_inner(&mut tokens, Delimiter::Brace, "Expected function body")?;

    Ok(OutpointArgs::new(
        OutpointExpr::from_tokens(url_expr)?,
        Some(config),
        Some(middleware),
        OutpointFunc::new(is_pub, fn_name, protocol, req_var_name, fn_cont, outer_attrs),
    ))
}

pub fn parse_attr(attr: TokenStream, args: TokenStream) -> Result<OutpointArgs, TokenStream> {
    let mut attr = into_peekable_iter(attr);
    let mut tokens = into_peekable_iter(args);

    let url_expr =
        expect_stream_before_comma_consume(&mut attr, false, "Expected URL Pattern")?;
    let mut middlewares = None;
    let mut config = None;
    if match_ident_consume(&mut attr, "middleware") {
        attr.next();
        middlewares = Some(expect_array_consume(&mut attr, "Expected an array for middleware")?);
    }
    if match_ident_consume(&mut attr, "config") {
        attr.next();
        config = Some(expect_array_consume(&mut attr, "Expected an array for config")?);
    }

    let outer_attrs = parse_outer_attrs(&mut tokens)?;
    let is_pub = match_ident_consume(&mut tokens, "pub");
    let _ = expect_ident_consume(&mut tokens, "fn", "Expected 'fn' keyword")?;
    let fn_name = match match_punct_consume(&mut tokens, "_") {
        true => {
            let random_name = format!("auto_generated_{}", random_alpha_string(8));
            Ident::new(&random_name, Span::call_site())
        }
        false => expect_any_ident(&mut tokens, "Expected function name")?,
    };
    let _ = expect_punct_consume(&mut tokens, "<", "Expected '<' after function name")?;
    let protocol = expect_any_ident(&mut tokens, "Expected protocol identifier after '<'")?;
    let _ = expect_punct_consume(&mut tokens, ">", "Expected '>' after protocol identifier")?;
    let mut group = into_peekable_iter(expect_group_consume_return_inner(
        &mut tokens,
        Delimiter::Parenthesis,
        "Expected function parameters inside parentheses",
    )?);
    let req_var_name = match expect_any_ident(&mut group, "Expected request variable name") {
        Ok(id) => id,
        Err(_) => Ident::new("req", Span::call_site()),
    };
    let fn_cont =
        expect_group_consume_return_inner(&mut tokens, Delimiter::Brace, "Expected function body")?;

    Ok(OutpointArgs::new(
        OutpointExpr::from_tokens(url_expr)?,
        config,
        middlewares,
        OutpointFunc::new(is_pub, fn_name, protocol, req_var_name, fn_cont, outer_attrs),
    ))
}
