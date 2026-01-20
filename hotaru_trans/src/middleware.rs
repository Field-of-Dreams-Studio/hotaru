use hotaru_lib::random::random_alpha_string;
use std::iter::Peekable;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::helper::*;

pub struct MWFunc {
    pub is_pub: bool,
    pub name: Ident,
    pub protocol: Ident,
    pub req_var_name: Ident,
    pub cont: TokenStream,
    pub attrs: OuterAttr,
}

impl MWFunc {
    pub fn new(
        is_pub: bool,
        name: Ident,
        protocol: Ident,
        req_var_name: Ident,
        cont: TokenStream,
        attrs: OuterAttr,
    ) -> Self {
        Self {
            name,
            protocol,
            cont,
            req_var_name,
            is_pub,
            attrs,
        }
    }

    pub fn expand(&self) -> TokenStream {
        // Helper: std::path::like builder
        fn path(parts: &[&str]) -> TokenStream {
            let mut ts = TokenStream::new();
            let mut first = true;
            for p in parts {
                if !first {
                    ts.extend(vec![
                        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    ]);
                }
                ts.extend(std::iter::once(TokenTree::Ident(Ident::new(
                    p,
                    Span::call_site(),
                ))));
                first = false;
            }
            ts
        }

        // #[...] pub struct <Name>;
        let mut struct_decl = TokenStream::new();
        struct_decl.extend(self.attrs.reform());

        // #[allow(non_camel_case_types)]
        let mut inner = TokenStream::new(); 
        inner.extend(vec![
            TokenTree::Ident(Ident::new("allow", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                let mut g = TokenStream::new();
                g.extend(std::iter::once(TokenTree::Ident(Ident::new(
                    "non_camel_case_types",
                    Span::call_site(),
                ))));
                g
            })),
        ]);
        struct_decl.extend(vec![
            TokenTree::Punct(Punct::new('#', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Bracket, inner)),
        ]);

        // pub struct name
        struct_decl.extend(vec![
            TokenTree::Ident(Ident::new("pub", Span::call_site())),
            TokenTree::Ident(Ident::new("struct", Span::call_site())),
            TokenTree::Ident(self.name.clone()),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);

        // impl AsyncMiddleware<<Protocol as Protocol>::Context> for Name { ... }
        let mut impl_head = TokenStream::new();
        impl_head.extend(vec![
            TokenTree::Ident(Ident::new("impl", Span::call_site())),
            TokenTree::Ident(Ident::new("AsyncMiddleware", Span::call_site())),
            // <<Protocol as Protocol>::Context>
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Ident(self.protocol.clone()),
            TokenTree::Ident(Ident::new("as", Span::call_site())),
            TokenTree::Ident(Ident::new("Protocol", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("Context", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Ident(Ident::new("for", Span::call_site())),
            TokenTree::Ident(self.name.clone()),
        ]);

        // fn as_any(&self) -> &dyn std::any::Any { self }
        let mut as_any_fn = TokenStream::new();
        {
            let mut params = TokenStream::new();
            params.extend(vec![
                // &self
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(Ident::new("self", Span::call_site())),
            ]);

            let mut ret_ty = TokenStream::new();
            ret_ty.extend(vec![
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Ident(Ident::new("dyn", Span::call_site())),
            ]);
            let any_path = path(&["std", "any", "Any"]);
            ret_ty.extend(any_path);

            let mut body = TokenStream::new();
            body.extend(vec![TokenTree::Ident(Ident::new(
                "self",
                Span::call_site(),
            ))]);

            as_any_fn.extend(vec![
                TokenTree::Ident(Ident::new("fn", Span::call_site())),
                TokenTree::Ident(Ident::new("as_any", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, params)),
                TokenTree::Punct(Punct::new('-', Spacing::Joint)),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, ret_ty)), // wrap type with () to attach '&dyn ...'
                TokenTree::Group(Group::new(Delimiter::Brace, body)),
            ]);
        }

        // fn return_self() -> Self where Self: Sized { Name }
        let mut return_self_fn = TokenStream::new();
        {
            let params = TokenStream::new();
            let mut body = TokenStream::new();
            body.extend(vec![TokenTree::Ident(self.name.clone())]);

            return_self_fn.extend(vec![
                TokenTree::Ident(Ident::new("fn", Span::call_site())),
                TokenTree::Ident(Ident::new("return_self", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, params)),
                TokenTree::Punct(Punct::new('-', Spacing::Joint)),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Self", Span::call_site())),
                TokenTree::Ident(Ident::new("where", Span::call_site())),
                TokenTree::Ident(Ident::new("Self", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Sized", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Brace, body)),
            ]);
        }

        // fn handle<'a>( &'a self, context: Protocol, next: Box<dyn Fn(Protocol) -> Pin<Box<dyn Future<Output=Protocol> + Send>> + Send + Sync + 'static>, ) -> Pin<Box<dyn Future<Output=Protocol> + Send + 'static>> { Box::pin(async move { let mut req = context; <user body> }) }
        let mut handle_fn = TokenStream::new();
        {
            // Params: &'a self, context: Protocol, next: Box<dyn Fn(Protocol) -> Pin<Box<dyn Future<Output=Protocol> + Send>> + Send + Sync + 'static>
            let mut params = TokenStream::new();
            // &'a self
            params.extend(vec![
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Punct(Punct::new('\'', Spacing::Joint)),
                TokenTree::Ident(Ident::new("a", Span::call_site())),
                TokenTree::Ident(Ident::new("self", Span::call_site())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            ]);
            // context: <Protocol as Protocol>::Context
            params.extend(vec![
                TokenTree::Ident(Ident::new("context", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                // <Protocol as Protocol>::Context
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                TokenTree::Ident(self.protocol.clone()),
                TokenTree::Ident(Ident::new("as", Span::call_site())),
                TokenTree::Ident(Ident::new("Protocol", Span::call_site())),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Context", Span::call_site())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            ]);
            // next: Box< ... >
            let mut next_ty = TokenStream::new();
            // Box
            next_ty.extend(path(&["Box"]));
            // <
            next_ty.extend(std::iter::once(TokenTree::Punct(Punct::new(
                '<',
                Spacing::Alone,
            ))));
            // dyn Fn(Protocol) -> std::pin::Pin<Box<dyn std::future::Future<Output = Protocol> + Send>>
            {
                // dyn
                next_ty.extend(std::iter::once(TokenTree::Ident(Ident::new(
                    "dyn",
                    Span::call_site(),
                ))));
                // Fn
                next_ty.extend(std::iter::once(TokenTree::Ident(Ident::new(
                    "Fn",
                    Span::call_site(),
                ))));
                // (<Protocol as Protocol>::Context)
                next_ty.extend(std::iter::once(TokenTree::Group(Group::new(
                    Delimiter::Parenthesis,
                    {
                        let mut g = TokenStream::new();
                        // <Protocol as Protocol>::Context
                        g.extend(vec![
                            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                            TokenTree::Ident(self.protocol.clone()),
                            TokenTree::Ident(Ident::new("as", Span::call_site())),
                            TokenTree::Ident(Ident::new("Protocol", Span::call_site())),
                            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                            TokenTree::Ident(Ident::new("Context", Span::call_site())),
                        ]);
                        g
                    },
                ))));
                // ->
                next_ty.extend(vec![
                    TokenTree::Punct(Punct::new('-', Spacing::Joint)),
                    TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                ]);
                // std::pin::Pin<Box<dyn std::future::Future<Output = Protocol> + Send>>
                let mut pin_box = TokenStream::new();
                // std::pin::Pin
                pin_box.extend(path(&["std", "pin", "Pin"]));
                pin_box.extend(std::iter::once(TokenTree::Punct(Punct::new(
                    '<',
                    Spacing::Alone,
                ))));
                // Box<dyn Future<Output=Protocol> + Send>
                let mut inner = TokenStream::new();
                inner.extend(path(&["Box"]));
                inner.extend(std::iter::once(TokenTree::Punct(Punct::new(
                    '<',
                    Spacing::Alone,
                ))));
                // dyn std::future::Future<Output = Protocol> + Send
                let mut dyn_future = TokenStream::new();
                dyn_future.extend(vec![TokenTree::Ident(Ident::new("dyn", Span::call_site()))]);
                dyn_future.extend(path(&["std", "future", "Future"]));
                // <Output = <Protocol as Protocol>::Context>
                dyn_future.extend(vec![
                    TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("Output", Span::call_site())),
                    TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                    // <Protocol as Protocol>::Context
                    TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                    TokenTree::Ident(self.protocol.clone()),
                    TokenTree::Ident(Ident::new("as", Span::call_site())),
                    TokenTree::Ident(Ident::new("Protocol", Span::call_site())),
                    TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("Context", Span::call_site())),
                    TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                ]);
                // + Send
                dyn_future.extend(vec![
                    TokenTree::Punct(Punct::new('+', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("Send", Span::call_site())),
                ]);
                inner.extend(std::iter::once(TokenTree::Group(Group::new(
                    Delimiter::Parenthesis,
                    dyn_future,
                )))); // wrap to keep cohesion
                inner.extend(std::iter::once(TokenTree::Punct(Punct::new(
                    '>',
                    Spacing::Alone,
                ))));
                // close Pin< ... >
                pin_box.extend(std::iter::once(TokenTree::Group(Group::new(
                    Delimiter::Parenthesis,
                    inner,
                ))));
                pin_box.extend(std::iter::once(TokenTree::Punct(Punct::new(
                    '>',
                    Spacing::Alone,
                ))));
                next_ty.extend(pin_box);
            }
            // + Send + Sync + 'static
            next_ty.extend(vec![
                TokenTree::Punct(Punct::new('+', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Send", Span::call_site())),
                TokenTree::Punct(Punct::new('+', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Sync", Span::call_site())),
                TokenTree::Punct(Punct::new('+', Spacing::Alone)),
                TokenTree::Punct(Punct::new('\'', Spacing::Joint)),
                TokenTree::Ident(Ident::new("static", Span::call_site())),
            ]);
            // >
            next_ty.extend(std::iter::once(TokenTree::Punct(Punct::new(
                '>',
                Spacing::Alone,
            ))));
            // next: <next_ty>
            params.extend(vec![
                TokenTree::Ident(Ident::new("next", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, next_ty)),
            ]);

            // Return type: -> std::pin::Pin<Box<dyn std::future::Future<Output=Protocol> + Send + 'static>>
            let mut ret_ty = TokenStream::new();
            // std::pin::Pin
            ret_ty.extend(path(&["std", "pin", "Pin"]));
            ret_ty.extend(std::iter::once(TokenTree::Punct(Punct::new(
                '<',
                Spacing::Alone,
            ))));
            // Box<dyn Future<Output=Protocol> + Send + 'static>
            let mut inner = TokenStream::new();
            inner.extend(path(&["Box"]));
            inner.extend(std::iter::once(TokenTree::Punct(Punct::new(
                '<',
                Spacing::Alone,
            ))));
            let mut dyn_future = TokenStream::new();
            dyn_future.extend(vec![TokenTree::Ident(Ident::new("dyn", Span::call_site()))]);
            dyn_future.extend(path(&["std", "future", "Future"]));
            dyn_future.extend(vec![
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Output", Span::call_site())),
                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                // <Protocol as Protocol>::Context
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                TokenTree::Ident(self.protocol.clone()),
                TokenTree::Ident(Ident::new("as", Span::call_site())),
                TokenTree::Ident(Ident::new("Protocol", Span::call_site())),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Context", Span::call_site())),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Punct(Punct::new('+', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Send", Span::call_site())),
                TokenTree::Punct(Punct::new('+', Spacing::Alone)),
                TokenTree::Punct(Punct::new('\'', Spacing::Joint)),
                TokenTree::Ident(Ident::new("static", Span::call_site())),
            ]);
            inner.extend(std::iter::once(TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                dyn_future,
            ))));
            inner.extend(std::iter::once(TokenTree::Punct(Punct::new(
                '>',
                Spacing::Alone,
            ))));
            ret_ty.extend(std::iter::once(TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                inner,
            ))));
            ret_ty.extend(std::iter::once(TokenTree::Punct(Punct::new(
                '>',
                Spacing::Alone,
            ))));

            // body: Box::pin(async move { let mut <req> = context; <user code> })
            let mut body = TokenStream::new();
            // Box::pin(
            body.extend(path(&["Box", "pin"]));
            body.extend(std::iter::once(TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                {
                    // async move { ... }
                    let mut async_move = TokenStream::new();
                    async_move.extend(vec![
                        TokenTree::Ident(Ident::new("async", Span::call_site())),
                        TokenTree::Ident(Ident::new("move", Span::call_site())),
                    ]);
                    // { let mut <req> = context; <fn body> }
                    let mut block = TokenStream::new();
                    // let mut <req> = context;
                    block.extend(vec![
                        TokenTree::Ident(Ident::new("let", Span::call_site())),
                        TokenTree::Ident(Ident::new("mut", Span::call_site())),
                        TokenTree::Ident(self.req_var_name.clone()),
                        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("context", Span::call_site())),
                        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                    ]);
                    // user body
                    block.extend(self.cont.clone());
                    async_move.extend(std::iter::once(TokenTree::Group(Group::new(
                        Delimiter::Brace,
                        block,
                    ))));
                    async_move
                },
            ))));

            handle_fn.extend(vec![
                TokenTree::Ident(Ident::new("fn", Span::call_site())),
                TokenTree::Ident(Ident::new("handle", Span::call_site())),
                // Lifetimes: <'a>
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                TokenTree::Punct(Punct::new('\'', Spacing::Joint)),
                TokenTree::Ident(Ident::new("a", Span::call_site())),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                // (params)
                TokenTree::Group(Group::new(Delimiter::Parenthesis, params)),
                // ->
                TokenTree::Punct(Punct::new('-', Spacing::Joint)),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                // return type
                TokenTree::Group(Group::new(Delimiter::Parenthesis, ret_ty)),
                // { body }
                TokenTree::Group(Group::new(Delimiter::Brace, body)),
            ]);
        }

        // impl body { as_any .. return_self .. handle .. }
        let mut impl_body = TokenStream::new();
        impl_body.extend(as_any_fn);
        impl_body.extend(return_self_fn);
        // dbg!(handle_fn.clone().to_string());
        impl_body.extend(handle_fn);

        // Final: struct + impl ...
        let mut out = TokenStream::new();
        out.extend(struct_decl);
        out.extend({
            let mut impl_block = TokenStream::new();
            impl_block.extend(impl_head);
            impl_block.extend(std::iter::once(TokenTree::Group(Group::new(
                Delimiter::Brace,
                impl_body,
            ))));
            impl_block
        });
        out
    }
}

pub fn parse_trans(input: TokenStream) -> Result<MWFunc, TokenStream> {
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
pub fn parse_semi_trans_or_attr(input: TokenStream) -> Result<MWFunc, TokenStream> {
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
