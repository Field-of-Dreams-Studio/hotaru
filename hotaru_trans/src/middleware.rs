use hotaru_lib::random::random_alpha_string;
use std::iter::Peekable;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::helper::generate_compile_error; 

struct MiddleWare {
    pub is_pub: bool,
    pub name: Ident,
    pub protocol: Ident, 
    pub req_var_name: Ident, 
    pub cont: TokenStream,
    pub attrs: Vec<TokenStream>,
}

impl MiddleWare {
    pub fn new(
        is_pub: bool,
        name: Ident,
        protocol: Ident,
        req_var_name: Ident, 
        cont: TokenStream,
        attrs: Vec<TokenStream>,
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

    pub fn parse(input: TokenStream) -> Result<Self, TokenStream> {
        fn parse_outer_attrs(
            tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
        ) -> Result<Vec<TokenStream>, TokenStream> {
            let mut attrs = Vec::new();

            loop {
                match tokens.peek() {
                    Some(TokenTree::Punct(p)) if p.as_char() == '#' => {
                        // consume '#'
                        tokens.next();

                        match tokens.next() {
                            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket => {
                                // Reject inner attributes: #![...]
                                let mut inside = g.stream().into_iter().peekable();
                                if let Some(TokenTree::Punct(p)) = inside.peek() {
                                    if p.as_char() == '!' {
                                        return Err(generate_compile_error(
                                            g.span(),
                                            "inner attributes (#![...] ) are not supported here",
                                        ));
                                    }
                                }

                                // Rebuild as "#[ ... ]" so we can re-emit verbatim later
                                let mut attr = TokenStream::new();
                                attr.extend(std::iter::once(TokenTree::Punct(Punct::new(
                                    '#',
                                    Spacing::Alone,
                                ))));
                                attr.extend(std::iter::once(TokenTree::Group(Group::new(
                                    Delimiter::Bracket,
                                    g.stream(),
                                ))));
                                attrs.push(attr);
                            }
                            Some(tt) => {
                                return Err(generate_compile_error(
                                    tt.span(),
                                    "expected attribute group after '#'",
                                ));
                            }
                            None => {
                                return Err(generate_compile_error(
                                    Span::call_site(),
                                    "expected attribute group after '#'",
                                ));
                            }
                        }
                    }
                    _ => break,
                }
            }

            Ok(attrs)
        }

        /// Return: (is_pub, is_fn_style, Ident) 
        fn process_name(
            tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
        ) -> Result<(bool, bool, Ident), TokenStream> {
            let mut is_pub = false;
            match tokens.peek() {
                Some(TokenTree::Ident(ident)) if ident.to_string() == "pub" => {
                    tokens.next();
                    is_pub = true;
                }
                Some(_) => {}
                None => {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        "Expected middleware name but found EOF",
                    ));
                }
            };

            let mut is_fn_style = false;
            match tokens.peek() {
                Some(TokenTree::Ident(ident)) if ident.to_string() == "fn" => {
                    tokens.next();
                    is_fn_style = true;
                }
                _ => {}
            }

            match tokens.next() {
                Some(TokenTree::Ident(ident)) if ident.to_string() == "_" => {
                    return Ok((
                        is_pub,
                        is_fn_style,
                        Ident::new(&random_alpha_string(32), Span::call_site()),
                    ));
                }
                Some(TokenTree::Ident(ident)) => Ok((is_pub, is_fn_style, ident)),
                Some(token) => Err(generate_compile_error(
                    token.span(),
                    &format!(
                        "Expected middleware name or `fn` keyword, but found {}",
                        token.to_string()
                    ),
                )),
                None => Err(generate_compile_error(
                    Span::call_site(),
                    "Expected middleware name or `fn` keyword but found EOF",
                )),
            }
        }

        fn process_protocol(
            tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
        ) -> Result<Ident, TokenStream> {
            match tokens.next() {
                Some(TokenTree::Punct(punct)) if punct.as_char() == '<' => {}
                Some(tt) => {
                    return Err(generate_compile_error(
                        tt.span(),
                        "Expect '< PROTOCOL >' around the protocol type after the middleware name",
                    ));
                }
                None => {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        "Expoect some tokens after middleware name",
                    ));
                }
            }

            let protocol;
            match tokens.next() {
                Some(TokenTree::Ident(ident)) => protocol = ident,
                Some(tt) => {
                    return Err(generate_compile_error(
                        tt.span(),
                        "Protocol Identifier is Expected",
                    ));
                }
                None => {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        "Expected Something",
                    ));
                }
            }

            match tokens.next() {
                Some(TokenTree::Punct(punct)) if punct.as_char() == '>' => {}
                Some(tt) => {
                    return Err(generate_compile_error(
                        tt.span(),
                        "Expect '>' after the protocol type",
                    ));
                }
                None => {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        "Expoect some tokens after middleware name",
                    ));
                }
            }

            Ok(protocol)
        }

        /// fn middleware_name(req: Protocol)
        /// Return: (Protocol, req_var_name) 
        fn process_arguments(
            tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
        ) -> Result<(Ident, Ident), TokenStream> {
            match tokens.next() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
                    let mut inside_tokens = group.stream().into_iter().peekable();

                    let req_var_name = match inside_tokens.next() {
                        Some(TokenTree::Ident(ident)) => ident,
                        Some(tt) => {
                            return Err(generate_compile_error(
                                tt.span(),
                                "Expected request variable name",
                            ));
                        }
                        None => {
                            return Err(generate_compile_error(
                                Span::call_site(),
                                "Expected something inside the parentheses",
                            ));
                        }
                    };

                    match inside_tokens.next() {
                        Some(TokenTree::Punct(punct)) if punct.as_char() == ':' => {}
                        Some(tt) => {
                            return Err(generate_compile_error(
                                tt.span(),
                                "Expected ':' after request variable name",
                            ));
                        }
                        None => {
                            return Err(generate_compile_error(
                                Span::call_site(),
                                "Expected ':' after request variable name",
                            ));
                        }
                    };

                    let protocol = match inside_tokens.next() {
                        Some(TokenTree::Ident(ident)) => ident,
                        Some(tt) => {
                            return Err(generate_compile_error(
                                tt.span(),
                                "Expected Protocol type after ':'",
                            ));
                        }
                        None => {
                            return Err(generate_compile_error(
                                Span::call_site(),
                                "Expected Protocol type after ':'",
                            ));
                        }
                    };

                    if let Some(tt) = inside_tokens.next() {
                        return Err(generate_compile_error(
                            tt.span(),
                            "Unexpected token inside parentheses",
                        ));
                    }

                    Ok((protocol, req_var_name))
                }
                Some(_) => Err(generate_compile_error(
                    Span::call_site(),
                    "Expected function arguments in parentheses",
                )),
                None => Err(generate_compile_error(
                    Span::call_site(),
                    "Expected function arguments after middleware name",
                )),
            }
        }

        fn process_func_content(
            tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
        ) -> Result<TokenStream, TokenStream> {
            match tokens.next() {
                Some(TokenTree::Group(group)) => {
                    if group.delimiter() == Delimiter::Brace {
                        Ok(group.stream())
                    } else {
                        Err(generate_compile_error(group.span(), "Expect middleware content"))
                    }
                }
                Some(tt) => Err(generate_compile_error(tt.span(), "Expect middleware content")),
                None => Err(generate_compile_error(
                    Span::call_site(),
                    "Expect some tokens",
                )),
            }
        }

        let mut tokens = input.into_iter().peekable();
        let attrs = parse_outer_attrs(&mut tokens)?;
        let (is_pub, fn_style, name) = process_name(&mut tokens)?;
        let (protocol, req_var_name) = if fn_style {
            process_arguments(&mut tokens)?
        } else {
            let protocol = process_protocol(&mut tokens)?;
            (protocol, Ident::new("req", Span::call_site()))
        };
        let cont = process_func_content(&mut tokens)?;

        Ok(Self::new(
            is_pub,
            name,
            protocol,
            req_var_name,
            cont,
            attrs,
        ))
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
        for a in &self.attrs {
            struct_decl.extend(a.clone());
        }
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

