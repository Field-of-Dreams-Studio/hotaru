use hotaru_lib::random::random_alpha_string;
use std::iter::Peekable;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::helper::generate_compile_error; 
use crate::ctor::gen_ctor; 

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

/// TODO 
/// Parse the attribute input into UrlAttr 
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

/// TODO 
/// Parse the function definition into UrlFunc 
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

