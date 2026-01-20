use hotaru_lib::random::random_alpha_string;
use std::iter::Peekable;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::{helper::*, outer_attr}; 
use crate::ctor::gen_ctor; 

/// Arguments for the `url` macro.
pub struct UrlArgs {
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

    pub fn expand(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        tokens.extend(self.op.generate_function());
        tokens.extend(self.op.wrapper_function());
        tokens.extend(self.reg_func()); 
        tokens
    } 
}

pub struct UrlFunc {
    pub is_pub: bool,
    pub fn_name: Ident,
    pub protocol: Ident,
    pub req_var_name: Ident, 
    pub fn_cont: TokenStream,
    pub attrs: OuterAttr,
}

impl UrlFunc {
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

/// Parse the attribute input into UrlAttr 
pub fn parse_trans(args: TokenStream) -> Result<UrlArgs, TokenStream> {  
    /// Parse the function definition into UrlFunc 
    fn parse_inner(tokens: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Result<UrlFunc, TokenStream> {
        let attrs = parse_outer_attrs(tokens)?; 
        let mut is_pub = match_ident_consume(tokens, "pub");  
        let mut fn_name = match match_punct_consume(tokens, "_"){ 
            true => {
                let random_name = format!("auto_generated_{}", random_alpha_string(8));
                Ident::new(&random_name, Span::call_site()) 
            }, 
            false => expect_any_ident(tokens, "Expected function name")? 
        }; 
        let _ = expect_punct_consume(tokens, "<", "Expected '<' after function name")?; 
        let protocol = expect_any_ident(tokens, "Expected protocol identifier after '<'")?;
        let _ = expect_punct_consume(tokens, ">", "Expected '>' after protocol identifier")?; 
        let fn_cont = expect_group_consume_return_inner(tokens, Delimiter::Brace, "Expected function body inside braces")?; 

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
    let url_expr = expect_stream_before_comma_consume(&mut tokens, true, "Expected a comma after the operations")?; 

    let mut middlewares = None;
    let mut config = None; 

    if match_ident_consume(&mut tokens, "middleware") { 
        tokens.next(); // Consume the `=` 
        middlewares = Some(expect_array_consume(&mut tokens, "Expected an array for middleware")?); 
    } 

    if match_ident_consume(&mut tokens, "config") { 
        tokens.next(); // Consume the `=`  
        config = Some(expect_array_consume(&mut tokens, "Expected an array for config")?); 
    } 

    return Ok(UrlArgs::new(
        url_expr,
        config,
        middlewares,
        parse_inner(&mut tokens)?,
    ));
} 

/// Expect to be in the following format: 
/// #[endpoint] 
/// #[url(APP.url("..."))] 
/// #[config([ ... ])] // Optional 
/// #[middleware([ ... ])] // Optional 
/// pub fn endpoint_name(req: Protocol) { 
///    ... 
/// }
pub fn parse_semi_trans(args: TokenStream) -> Result<UrlArgs, TokenStream> {  
    let mut tokens = into_peekable_iter(args); 
    
    let mut outer_attrs = parse_outer_attrs(&mut tokens)?; 
    let url_expr_raw = outer_attrs.remove("url").ok_or(generate_compile_error(Span::call_site(), "Missing required 'url' attribute"))?; 
    let url_expr = OuterAttr::get_inners(url_expr_raw, "Expected url(...)")?; 
    let config = outer_attrs.remove("config").map(|ts| expect_array_consume(&mut into_peekable_iter(OuterAttr::get_inners(ts, "Expected config([...])")?), "Expected an array for config")).unwrap_or(Ok(vec![]))?; 
    let middleware = outer_attrs.remove("middleware").map(|ts| expect_array_consume(&mut into_peekable_iter(OuterAttr::get_inners(ts, "Expected middleware([...])")?), "Expected an array for middleware")).unwrap_or(Ok(vec![]))?; 
    
    let is_pub = match_ident_consume(&mut tokens, "pub"); 
    let _ = expect_ident_consume(&mut tokens, "fn", "Expected 'fn' keyword for function definition")?; 
    let fn_name = match match_punct_consume(&mut tokens, "_"){ 
        true => {
            let random_name = format!("auto_generated_{}", random_alpha_string(8));
            Ident::new(&random_name, Span::call_site()) 
        }, 
        false => expect_any_ident(&mut tokens, "Expected function name")? 
    }; 
    let _ = expect_punct_consume(&mut tokens, "<", "Expected '<' after function name")?; 
    let protocol = expect_any_ident(&mut tokens, "Expected protocol identifier after '<'")?;
    let _ = expect_punct_consume(&mut tokens, ">", "Expected '>' after protocol identifier")?;
    let mut group = into_peekable_iter(expect_group_consume_return_inner(&mut tokens, Delimiter::Parenthesis, "Expected function parameters inside parentheses")?); 
    let req_var_name = match expect_any_ident(&mut group, "Expected request variable name as first parameter") { 
        Ok(id) => id, 
        Err(_) => Ident::new("req", Span::call_site()), 
    }; 
    let fn_cont = expect_group_consume_return_inner(&mut tokens, Delimiter::Brace, "Expected function body inside braces")?; 

    return Ok(UrlArgs::new(
        url_expr,
        Some(config),
        Some(middleware),
        UrlFunc::new(
            is_pub, 
            fn_name, 
            protocol,
            req_var_name,
            fn_cont,
            outer_attrs
        ),
    )); 
} 
