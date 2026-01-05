use hotaru_lib::random::random_alpha_string;
use std::iter::Peekable;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

/// Generate constructor attribute for automatic registration
/// By default uses built-in #[hotaru::hotaru_meta::ctor]
/// Enable "external-ctor" feature to use #[ctor::ctor] from ctor crate instead
fn gen_ctor() -> TokenStream {
    #[cfg(feature = "external-ctor")]
    {
        // Use external ctor crate: #[ctor::ctor]
        let mut ctor_macro = TokenStream::new();
        ctor_macro.extend(vec![
            TokenTree::Punct(Punct::new('#', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Bracket, {
                let mut attr = TokenStream::new();
                attr.extend(vec![
                    TokenTree::Ident(Ident::new("ctor", Span::call_site())),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("ctor", Span::call_site())),
                ]);
                attr
            })),
        ]);
        ctor_macro
    }

    #[cfg(not(feature = "external-ctor"))]
    {
        // Use built-in hotaru::hotaru_meta::ctor
        let mut ctor_macro = TokenStream::new();
        ctor_macro.extend(vec![
            TokenTree::Punct(Punct::new('#', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Bracket, {
                let mut attr = TokenStream::new();
                attr.extend(vec![
                    TokenTree::Ident(Ident::new("hotaru", Span::call_site())),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("hotaru_meta", Span::call_site())),
                    TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                    TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("ctor", Span::call_site())),
                ]);
                attr
            })),
        ]);
        ctor_macro
    }
}

/// Arguments for the `url` macro.
struct UrlArgs {
    pub url_expr: TokenStream,
    pub config: Option<Vec<TokenStream>>,
    pub middlewares: Option<Vec<TokenStream>>,
    pub op: UrlFunc,
}

impl UrlArgs {
    pub fn new(
        url_expr: TokenStream,
        config: Option<Vec<TokenStream>>,
        middlewares: Option<Vec<TokenStream>>,
        op: UrlFunc,
    ) -> Self {
        UrlArgs {
            url_expr,
            config,
            middlewares,
            op,
        }
    }

    pub fn parse(args: TokenStream) -> Result<Self, TokenStream> {
        fn split_top_level_until_comma(input: TokenStream) -> Vec<TokenStream> {
            let mut tokens = input.into_iter();
            let mut vec = vec![];
            let mut next_stream = TokenStream::new();
            loop {
                match tokens.next() {
                    Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                        vec.push(next_stream);
                        next_stream = TokenStream::new();
                    }
                    Some(tt) => {
                        next_stream.extend(std::iter::once(tt));
                    }
                    None => break,
                }
            }
            vec.push(next_stream);
            vec
        }

        fn expect_punct(
            tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
            ch: char,
        ) -> Result<(), TokenStream> {
            match tokens.next() {
                Some(TokenTree::Punct(p)) if p.as_char() == ch => Ok(()),
                Some(tt) => Err(generate_compile_error(
                    tt.span(),
                    &format!("expected '{}'", ch),
                )),
                None => Err(generate_compile_error(
                    Span::call_site(),
                    &format!("expected '{}'", ch),
                )),
            }
        }

        fn parse_array(
            tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
        ) -> Result<Vec<TokenStream>, TokenStream> {
            match tokens.peek() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
                    let mut array = Vec::new();
                    let mut current = TokenStream::new();
                    let mut inside_tokens = group.stream().into_iter();
                    loop {
                        match inside_tokens.next() {
                            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {
                                array.push(current);
                                current = TokenStream::new();
                            }
                            Some(token) => current.extend(std::iter::once(token)),
                            None => {
                                array.push(current);
                                break;
                            }
                        }
                    }
                    // dbg!(array.clone());
                    return Ok(array);
                }
                Some(tt) => {
                    return Err(generate_compile_error(
                        tt.span(),
                        "Expect an [] after the attribute",
                    ));
                }
                None => {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        "Expect something after an attribute",
                    ));
                }
            }
        }

        let mut tokens = split_top_level_until_comma(args);
        let op = match tokens.pop() {
            Some(ts) => ts,
            None => {
                return Err(generate_compile_error(
                    Span::call_site(),
                    "The last token should be the operations",
                ));
            }
        };

        let mut tokens = tokens.into_iter();

        let url_expr = match tokens.next() {
            Some(ts) => ts,
            None => {
                return Err(generate_compile_error(
                    Span::call_site(),
                    "There should be a token repr the endpoint",
                ));
            }
        };

        let mut middlewares = None;
        let mut config = None;

        // Read the Middleware or Configs, middleware and configs should be
        // #[url(<UrlExpr>, middleware=[...], config=[...])]
        while let Some(ts) = tokens.next() {
            let mut internal_tokens = ts.into_iter().peekable();
            match internal_tokens.next() {
                Some(TokenTree::Ident(ident)) if ident.to_string() == "middleware" => {
                    internal_tokens.next(); // Consume the `=` 
                    middlewares = Some(parse_array(&mut internal_tokens)?);
                }
                Some(TokenTree::Ident(ident)) if ident.to_string() == "config" => {
                    internal_tokens.next(); // Consume the `=`  
                    config = Some(parse_array(&mut internal_tokens)?);
                }
                Some(tt) => {
                    return Err(generate_compile_error(
                        tt.span(),
                        "Expect `middleware` or `config`",
                    ));
                }
                _ => {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        "Expect `middleware` or `config`",
                    ));
                }
            }
        }

        return Ok(Self::new(
            url_expr,
            config,
            middlewares,
            UrlFunc::parse(op)?,
        ));
    }

    pub fn reg_func(&self) -> TokenStream {
        // Generate constructor attributes using gen_ctor()
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

        // let mut endpoint = { <url-expr> };
        // endpoint.set_method(std::sync::Arc::new(__wrapper_xxx));
        
        // Modify url_expr to inject the protocol type parameter
        let mut modified_url_expr = TokenStream::new();
        let mut url_tokens = self.url_expr.clone().into_iter().peekable();
        
        // Process the url expression to inject type parameter after .url
        while let Some(token) = url_tokens.next() {
            modified_url_expr.extend(std::iter::once(token.clone()));
            
            // Check if this is the "url" identifier
            if let TokenTree::Ident(ident) = &token {
                if ident.to_string() == "url" {
                    // Inject the type parameter ::<Protocol, _>
                    modified_url_expr.extend(vec![
                        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                        TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                        TokenTree::Ident(self.op.protocol.clone()),
                        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("_", Span::call_site())),
                        TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                    ]);
                }
            }
        }
        
        let mut cont = TokenStream::new();
        cont.extend(vec![
            TokenTree::Ident(Ident::new("let", Span::call_site())),
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
            TokenTree::Ident(Ident::new("endpoint", Span::call_site())),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Brace, modified_url_expr)),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            TokenTree::Ident(Ident::new("endpoint", Span::call_site())),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("set_method", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, arc_func)),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);

        // Inserting configs
        if let Some(configs) = self.config.clone() {
            for expr in configs {
                cont.extend(vec![
                    TokenTree::Ident(Ident::new("endpoint", Span::call_site())),
                    TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("set_params", Span::call_site())),
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, expr)),
                    TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                ])
            }
        }

        if let Some(mws) = self.middlewares.clone() {
            // Middleware inheritance implementation
            // This section handles the special ".." token which inherits middleware from the protocol's root URL
            
            // let mut middlewares: Vec<std::sync::Arc<dyn hotaru::hotaru_core::app::middleware::AsyncMiddleware<Protocol> + 'static>> = vec![];
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
                TokenTree::Punct(Punct::new('>', Spacing::Alone)), // close Arc<
                TokenTree::Punct(Punct::new('>', Spacing::Alone)), // close Vec<
                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                TokenTree::Ident(Ident::new("vec", Span::call_site())),
                TokenTree::Punct(Punct::new('!', Spacing::Alone)),
                TokenTree::Group(Group::new(Delimiter::Bracket, TokenStream::new())),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            ]);

            cont.extend(mw_decl);

            // Process middleware array, handling both regular middleware and the special ".." inheritance token
            // The ".." token inherits all middleware from the protocol's root URL handler
            // It can appear at any position in the array to control ordering:
            // - [.., LocalMw] - inherited middleware first, then local
            // - [LocalMw, ..] - local middleware first, then inherited
            // - [LocalMw1, .., LocalMw2] - LocalMw1, then inherited, then LocalMw2
            
            // Push each middleware individually to allow Arc<Concrete> -> Arc<dyn Trait> coercion.
            for expr in mws {
                // Check if this token is ".." (middleware inheritance marker)
                let is_dots = {
                    let tokens: Vec<TokenTree> = expr.clone().into_iter().collect();
                    tokens.len() == 2 &&
                    matches!(tokens.get(0), Some(TokenTree::Punct(p)) if p.as_char() == '.') &&
                    matches!(tokens.get(1), Some(TokenTree::Punct(p)) if p.as_char() == '.')
                };
                
                if is_dots {
                    // Generate code to inherit middleware from protocol root at runtime
                    // Direct APP access approach for middleware inheritance
                    // This uses the APP() method directly instead of checking ancestor relationships
                    
                    // Generated code structure:
                    // {
                    //     let protocol_middlewares = APP.handler.get_protocol_middlewares::<Protocol>();
                    //     middlewares.extend(protocol_middlewares);
                    // }
                    
                    let mut inheritance_block = TokenStream::new();
                    
                    // Create the content of the scoped block  
                    let mut block_content = TokenStream::new();
                    
                    // let protocol_middlewares = APP.handler.get_protocol_middlewares::<Protocol>();
                    block_content.extend(vec![
                        TokenTree::Ident(Ident::new("let", Span::call_site())),
                        TokenTree::Ident(Ident::new("protocol_middlewares", Span::call_site())),
                        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("APP", Span::call_site())),
                        TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("handler", Span::call_site())),
                        TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("get_protocol_middlewares", Span::call_site())),
                        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                        TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                        TokenTree::Ident(self.op.protocol.clone()),
                        TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
                        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                    ]);
                    
                    // middlewares.extend(protocol_middlewares);
                    block_content.extend(vec![
                        TokenTree::Ident(Ident::new("middlewares", Span::call_site())),
                        TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("extend", Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                            let mut g = TokenStream::new();
                            g.extend(vec![
                                TokenTree::Ident(Ident::new("protocol_middlewares", Span::call_site())),
                            ]);
                            g
                        })),
                        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                    ]);
                    
                    inheritance_block.extend(vec![
                        TokenTree::Group(Group::new(Delimiter::Brace, block_content)),
                    ]);
                    
                    cont.extend(inheritance_block);
                } else {
                    // Regular middleware - use existing logic
                    let mut push_call = TokenStream::new();
                    // middlewares.push(std::sync::Arc::new(expr));
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

            // endpoint.set_middlewares(middlewares);
            cont.extend(vec![
                TokenTree::Ident(Ident::new("endpoint", Span::call_site())),
                TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                TokenTree::Ident(Ident::new("set_middlewares", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                    let mut g = TokenStream::new();
                    g.extend(std::iter::once(TokenTree::Ident(Ident::new(
                        "middlewares",
                        Span::call_site(),
                    ))));
                    g
                })),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            ]);
        }

        let mut tokens = TokenStream::new();
        tokens.extend(ctor_attrs);
        tokens.extend(vec![
            TokenTree::Ident(Ident::new("fn", Span::call_site())),
            TokenTree::Ident(Ident::new(
                &format!("__register_{}", &self.op.fn_name),
                Span::call_site(),
            )),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
            TokenTree::Group(Group::new(Delimiter::Brace, cont)),
        ]);

        tokens
    }
}

struct UrlFunc {
    pub is_pub: bool,
    pub fn_name: String,
    pub protocol: Ident,
    pub req_var_name: Ident, 
    pub fn_cont: TokenStream,
    pub attrs: Vec<TokenStream>,
}

impl UrlFunc {
    pub fn new(
        is_pub: bool,
        fn_name: String,
        protocol: Ident, 
        req_var_name: Ident, 
        fn_cont: TokenStream,
        attrs: Vec<TokenStream>,
    ) -> Self {
        Self {
            is_pub,
            fn_name,
            protocol,
            fn_cont,
            req_var_name, 
            attrs,
        }
    }

    pub fn parse(function: TokenStream) -> Result<Self, TokenStream> {
        /// Return: ((is_pub, is_fn_style, fn_name), TokenStream) 
        /// Second value = True if use (pub)? fn xxx(req: XXX) style, False if use (pub)? xxx <XXX> style
        fn process_name(
            tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
        ) -> Result<(bool, bool, String), TokenStream> {
            // Whether it is a pub endpoint or not 
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
                        "Expected Function Name but found EOF",
                    ));
                }
            }; 

            // Peek the function keyword and branch accordingly 
            let mut is_fn_style = false; 
            match tokens.peek() {
                Some(TokenTree::Ident(ident)) if ident.to_string() == "fn" => { 
                    tokens.next(); 
                    is_fn_style = true; 
                } 
                _ => {} 
            } 

            // Get the function name 
            match tokens.next() {
                Some(TokenTree::Ident(ident)) if ident.to_string() == "_" => {
                    return Ok((is_pub, is_fn_style, random_alpha_string(32)));
                }
                Some(TokenTree::Ident(ident)) => {
                    return Ok((is_pub, is_fn_style, ident.to_string()));
                }
                Some(token) => {
                    return Err(generate_compile_error(
                        token.span(),
                        &format!("Expected Function Name or `fn` Keyword, but found {}", token.to_string()),
                    ));
                }
                None => {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        "Expected Function Name or `fn` keyword. but found EOF",
                    ));
                }
            }
        }

        fn process_protocol(
            tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
        ) -> Result<Ident, TokenStream> {
            // Expect "<>" around the protocol type
            match tokens.next() {
                Some(TokenTree::Punct(punct)) if punct == '<' => {}
                Some(tt) => {
                    return Err(generate_compile_error(
                        tt.span(),
                        "Expect '< PROTOCOL >' around the protocol type after the function name",
                    ));
                }
                None => {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        "Expoect some tokens after function name",
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
                Some(TokenTree::Punct(punct)) if punct == '>' => {}
                Some(tt) => {
                    return Err(generate_compile_error(
                        tt.span(),
                        "Expect '>' after the protocol type",
                    ));
                }
                None => {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        "Expoect some tokens after function name",
                    ));
                }
            }

            return Ok(protocol);
        } 

        /// pub fn endpoint_name(req: Protocol) 
        /// Return: (Protocol, req_var_name) 
        fn process_arguments(
            tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
        ) -> Result<(Ident, Ident), TokenStream> { 
            match tokens.next() { 
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => { 
                    let mut inside_tokens = group.stream().into_iter().peekable(); 

                    // Expect the request variable name 
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

                    // Expect ':' 
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

                    // Expect Protocol type 
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

                    // Ensure no extra tokens inside parentheses
                    if let Some(tt) = inside_tokens.next() { 
                        return Err(generate_compile_error( 
                            tt.span(), 
                            "Unexpected token inside parentheses", 
                        )); 
                    } 

                    Ok((protocol, req_var_name))  
                }
                Some(_) => {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        "Expected function arguments in parentheses",
                    ));
                }
                None => {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        "Expected function arguments after function name",
                    ));
                } 
            }
        }

        fn process_func_content(
            tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
        ) -> Result<TokenStream, TokenStream> {
            match tokens.next() {
                Some(TokenTree::Group(group)) => {
                    if group.delimiter() == Delimiter::Brace {
                        return Ok(group.stream());
                    } else {
                        return Err(generate_compile_error(
                            group.span(),
                            "Expected Code Segment",
                        ));
                    }
                }
                Some(tt) => return Err(generate_compile_error(tt.span(), "Expected Code Segment")),
                None => {
                    return Err(generate_compile_error(
                        Span::call_site(),
                        "Expected Code Segment",
                    ));
                }
            }
        }

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
                                            "inner attributes (#![...]) are not supported here",
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

        let mut tokens = function.clone().into_iter().peekable();

        // Collect leading #[...] attributes
        let attrs = parse_outer_attrs(&mut tokens)?;

        let (is_pub, fn_style, fn_name) = process_name(&mut tokens)?; 
        let (protocol, req_var_name) = if fn_style { 
            process_arguments(&mut tokens)? 
        } else { 
            let protocol = process_protocol(&mut tokens)?;
            (protocol, Ident::new("req", Span::call_site()))
        };
        let func_cont = process_func_content(&mut tokens)?;

        Ok(Self::new(is_pub, fn_name, protocol, req_var_name, func_cont, attrs))
    }

    pub fn generate_function(&self) -> TokenStream {
        let mut arguments = TokenStream::new();
        arguments.extend(vec![
            TokenTree::Ident(self.req_var_name.clone()),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Punct(Punct::new('&', Spacing::Alone)),
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
            // <Protocol as Protocol>::Context
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

        // Re-emit captured attributes (includes #[doc = "..."] if provided)
        for a in &self.attrs {
            tokens.extend(a.clone());
        }

        if self.is_pub {
            tokens.extend(vec![TokenTree::Ident(Ident::new("pub", Span::call_site()))]);
        }
        tokens.extend(vec![
            TokenTree::Ident(Ident::new("async", Span::call_site())),
            TokenTree::Ident(Ident::new("fn", Span::call_site())),
            TokenTree::Ident(Ident::new(&self.fn_name, Span::call_site())),
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
            // <Protocol as Protocol>::Context
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
            TokenTree::Ident(Ident::new(&self.fn_name, Span::call_site())),
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
            TokenTree::Ident(self.req_var_name.clone()) 
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
            // <Protocol as Protocol>::Context
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

#[proc_macro]
pub fn endpoint(input: TokenStream) -> TokenStream {
    let url_args = match UrlArgs::parse(input) {
        Ok(args) => args,
        Err(err) => return err,
    };

    let mut token_stream = TokenStream::new();
    token_stream.extend(url_args.op.generate_function());
    token_stream.extend(url_args.op.wrapper_function());
    token_stream.extend(url_args.reg_func());

    token_stream
}

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

#[proc_macro]
pub fn middleware(input: TokenStream) -> TokenStream {
    let mw = match MiddleWare::parse(input) {
        Ok(mw) => mw,
        Err(err) => return err,
    };
    mw.expand()
}

fn generate_compile_error(span: proc_macro::Span, message: &str) -> TokenStream {
    let mut tokens = TokenStream::new();
    let ident = TokenTree::Ident(Ident::new("compile_error", span));
    let mut punct = TokenTree::Punct(Punct::new('!', Spacing::Alone));
    punct.set_span(span);
    let mut message = Literal::string(message);
    message.set_span(span);
    let mut message = TokenTree::Group(proc_macro::Group::new(
        proc_macro::Delimiter::Parenthesis,
        TokenStream::from(TokenTree::Literal(message)),
    ));
    message.set_span(span);
    let mut semi_column = Punct::new(';', Spacing::Alone);
    semi_column.set_span(span);
    let mut semi_column = TokenTree::Punct(semi_column);
    semi_column.set_span(span);
    tokens.extend(vec![ident, punct, message, semi_column]);
    tokens
}

/// Our own constructor attribute - works like #[ctor::ctor] but built-in
/// Generates platform-specific linker sections for automatic initialization
#[cfg(not(feature = "external-ctor"))]
#[proc_macro_attribute]
pub fn ctor(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let mut tokens = item.into_iter().peekable();

    // Collect the entire function
    let function_tokens: Vec<_> = tokens.collect();

    // Extract function name for generating unique static name
    let fn_name = function_tokens.iter()
        .skip_while(|t| !matches!(t, TokenTree::Ident(id) if id.to_string() == "fn"))
        .nth(1)
        .and_then(|t| if let TokenTree::Ident(id) = t { Some(id.to_string()) } else { None })
        .unwrap_or_else(|| "unknown".to_string());

    let mut output = TokenStream::new();

    // Re-emit the original function
    output.extend(function_tokens);

    // Generate platform-specific static variable with linker sections
    let mut static_decl = TokenStream::new();

    // #[allow(unsafe_code)] - suppress unsafe warnings like ctor does
    static_decl.extend(vec![
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, {
            let mut attr = TokenStream::new();
            attr.extend(vec![
                TokenTree::Ident(Ident::new("allow", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                    let mut inner = TokenStream::new();
                    inner.extend(vec![TokenTree::Ident(Ident::new("unsafe_code", Span::call_site()))]);
                    inner
                })),
            ]);
            attr
        })),
    ]);

    // #[used]
    static_decl.extend(vec![
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, {
            let mut attr = TokenStream::new();
            attr.extend(vec![TokenTree::Ident(Ident::new("used", Span::call_site()))]);
            attr
        })),
    ]);

    // #[cfg_attr(target_os = "linux", unsafe(link_section = ".init_array"))]
    static_decl.extend(vec![
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, {
            let mut attr = TokenStream::new();
            attr.extend(vec![
                TokenTree::Ident(Ident::new("cfg_attr", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                    let mut inner = TokenStream::new();
                    inner.extend(vec![
                        TokenTree::Ident(Ident::new("target_os", Span::call_site())),
                        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                        TokenTree::Literal(Literal::string("linux")),
                        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("unsafe", Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                            let mut unsafe_inner = TokenStream::new();
                            unsafe_inner.extend(vec![
                                TokenTree::Ident(Ident::new("link_section", Span::call_site())),
                                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                                TokenTree::Literal(Literal::string(".init_array")),
                            ]);
                            unsafe_inner
                        })),
                    ]);
                    inner
                })),
            ]);
            attr
        })),
    ]);

    // #[cfg_attr(target_vendor = "apple", unsafe(link_section = "__DATA,__mod_init_func"))]
    static_decl.extend(vec![
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, {
            let mut attr = TokenStream::new();
            attr.extend(vec![
                TokenTree::Ident(Ident::new("cfg_attr", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                    let mut inner = TokenStream::new();
                    inner.extend(vec![
                        TokenTree::Ident(Ident::new("target_vendor", Span::call_site())),
                        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                        TokenTree::Literal(Literal::string("apple")),
                        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("unsafe", Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                            let mut unsafe_inner = TokenStream::new();
                            unsafe_inner.extend(vec![
                                TokenTree::Ident(Ident::new("link_section", Span::call_site())),
                                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                                TokenTree::Literal(Literal::string("__DATA,__mod_init_func")),
                            ]);
                            unsafe_inner
                        })),
                    ]);
                    inner
                })),
            ]);
            attr
        })),
    ]);

    // #[cfg_attr(target_os = "windows", unsafe(link_section = ".CRT$XCU"))]
    static_decl.extend(vec![
        TokenTree::Punct(Punct::new('#', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Bracket, {
            let mut attr = TokenStream::new();
            attr.extend(vec![
                TokenTree::Ident(Ident::new("cfg_attr", Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                    let mut inner = TokenStream::new();
                    inner.extend(vec![
                        TokenTree::Ident(Ident::new("target_os", Span::call_site())),
                        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                        TokenTree::Literal(Literal::string("windows")),
                        TokenTree::Punct(Punct::new(',', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("unsafe", Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                            let mut unsafe_inner = TokenStream::new();
                            unsafe_inner.extend(vec![
                                TokenTree::Ident(Ident::new("link_section", Span::call_site())),
                                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                                TokenTree::Literal(Literal::string(".CRT$XCU")),
                            ]);
                            unsafe_inner
                        })),
                    ]);
                    inner
                })),
            ]);
            attr
        })),
    ]);

    // static __CTOR_<name>: extern "C" fn() = { extern "C" fn __wrapper() { <fn_name>() }; __wrapper };
    static_decl.extend(vec![
        TokenTree::Ident(Ident::new("static", Span::call_site())),
        TokenTree::Ident(Ident::new(&format!("__CTOR_{}", fn_name.to_uppercase()), Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("extern", Span::call_site())),
        TokenTree::Literal(Literal::string("C")),
        TokenTree::Ident(Ident::new("fn", Span::call_site())),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Brace, {
            let mut inner = TokenStream::new();
            inner.extend(vec![
                TokenTree::Ident(Ident::new("extern", Span::call_site())),
                TokenTree::Literal(Literal::string("C")),
                TokenTree::Ident(Ident::new("fn", Span::call_site())),
                TokenTree::Ident(Ident::new(&format!("__wrapper_{}", fn_name), Span::call_site())),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
                TokenTree::Group(Group::new(Delimiter::Brace, {
                    let mut call = TokenStream::new();
                    call.extend(vec![
                        TokenTree::Ident(Ident::new(&fn_name, Span::call_site())),
                        TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
                    ]);
                    call
                })),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                TokenTree::Ident(Ident::new(&format!("__wrapper_{}", fn_name), Span::call_site())),
            ]);
            inner
        })),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);

    output.extend(static_decl);
    output
}

/// Helper macro to generate lazy static declarations
/// Used by LApp!, LUrl!, and LPattern! macros
macro_rules! generate_lazy_static {
    ($type_name:expr) => {
        |input: TokenStream| -> TokenStream {
            let mut tokens = input.into_iter().peekable();

            // Parse identifier
            let ident = match tokens.next() {
                Some(TokenTree::Ident(i)) => i,
                _ => return generate_compile_error(Span::call_site(), "Expected identifier before '='"),
            };

            // Expect '='
            match tokens.next() {
                Some(TokenTree::Punct(p)) if p.as_char() == '=' => {},
                _ => return generate_compile_error(Span::call_site(), "Expected '=' after identifier"),
            };

            // Collect the rest as the expression
            let expr: TokenStream = tokens.collect();

            if expr.clone().into_iter().next().is_none() {
                return generate_compile_error(Span::call_site(), "Expected expression after '='");
            }

            // Generate: pub static IDENT: TYPE = Lazy::new(|| EXPR);
            let mut output = TokenStream::new();

            // pub static
            output.extend(vec![
                TokenTree::Ident(Ident::new("pub", Span::call_site())),
                TokenTree::Ident(Ident::new("static", Span::call_site())),
            ]);

            // IDENT
            output.extend(vec![TokenTree::Ident(ident)]);

            // : TYPE
            output.extend(vec![
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new($type_name, Span::call_site())),
            ]);

            // = Lazy::new
            output.extend(vec![
                TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                TokenTree::Ident(Ident::new("Lazy", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Joint)),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("new", Span::call_site())),
            ]);

            // (|| EXPR)
            let mut closure = TokenStream::new();
            closure.extend(vec![
                TokenTree::Punct(Punct::new('|', Spacing::Joint)),
                TokenTree::Punct(Punct::new('|', Spacing::Alone)),
            ]);
            closure.extend(expr);

            output.extend(vec![
                TokenTree::Group(Group::new(Delimiter::Parenthesis, closure)),
                TokenTree::Punct(Punct::new(';', Spacing::Alone)),
            ]);

            output
        }
    };
}

/// `LApp!` - Creates a lazy static App instance
///
/// # Usage
/// ```rust
/// LApp!(APP = App::new().build());
/// ```
///
/// # Expansion
/// ```rust
/// pub static APP: SApp = Lazy::new(|| App::new().build());
/// ```
#[allow(non_snake_case)]
#[proc_macro]
pub fn LApp(input: TokenStream) -> TokenStream {
    generate_lazy_static!("SApp")(input)
}

/// `LUrl!` - Creates a lazy static Url instance
///
/// # Usage
/// ```rust
/// LUrl!(HOME = Url::new("/"));
/// ```
///
/// # Expansion
/// ```rust
/// pub static HOME: SUrl<_> = Lazy::new(|| Url::new("/"));
/// ```
#[allow(non_snake_case)]
#[proc_macro]
pub fn LUrl(input: TokenStream) -> TokenStream {
    generate_lazy_static!("SUrl")(input)
}

/// `LPattern!` - Creates a lazy static PathPattern instance
///
/// # Usage
/// ```rust
/// LPattern!(PATTERN = PathPattern::new("/*"));
/// ```
///
/// # Expansion
/// ```rust
/// pub static PATTERN: SPattern = Lazy::new(|| PathPattern::new("/*"));
/// ```
#[allow(non_snake_case)]
#[proc_macro]
pub fn LPattern(input: TokenStream) -> TokenStream {
    generate_lazy_static!("SPattern")(input)
}
