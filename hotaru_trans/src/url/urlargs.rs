use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::ctor::gen_ctor;
use crate::url::url_func::UrlFunc;
use crate::url::urlexpr::UrlExpr; 

/// Arguments for the `url` macro.
pub struct UrlArgs {
    pub url_expr: UrlExpr,
    pub config: Option<Vec<TokenStream>>,
    pub middlewares: Option<Vec<TokenStream>>,
    pub op: UrlFunc,
}

impl UrlArgs {
    pub fn new(
        url_expr: UrlExpr,
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

        let mut cont = TokenStream::new(); 

        // Generate empty configs (Params) 
        // let mut params = hotaru::akari::extensions::ParamsClone::default(); 
        cont.extend(vec![
            TokenTree::Ident(Ident::new("let", Span::call_site())),
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
            TokenTree::Ident(Ident::new("params", Span::call_site())),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
            TokenTree::Ident(Ident::new("hotaru", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("akari", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("extensions", Span::call_site())), 
            TokenTree::Punct(Punct::new(':', Spacing::Joint)), 
            TokenTree::Punct(Punct::new(':', Spacing::Alone)), 
            TokenTree::Ident(Ident::new("ParamsClone", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("default", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)), 
        ]); 

        // Inserting configs 
        // params.set(value) 
        if let Some(configs) = self.config.clone() {
            for expr in configs {
                cont.extend(vec![
                    TokenTree::Ident(Ident::new("params", Span::call_site())),
                    TokenTree::Punct(Punct::new('.', Spacing::Alone)),
                    TokenTree::Ident(Ident::new("set", Span::call_site())),
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, expr)),
                    TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                ])
            }
        } 

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

        // let mut binding = hotaru::hotaru_core::executable::ExecutableBinding::new().with_handler(std::sync::Arc::new(__wrapper_xxx)); 
        cont.extend(vec![
            TokenTree::Ident(Ident::new("let", Span::call_site())),
            TokenTree::Ident(Ident::new("mut", Span::call_site())),
            TokenTree::Ident(Ident::new("binding", Span::call_site())),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
            TokenTree::Ident(Ident::new("hotaru", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("hotaru_core", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("executable", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("ExecutableBinding", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("new", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
            TokenTree::Punct(Punct::new('.', Spacing::Alone)),
            TokenTree::Ident(Ident::new("with_handler", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, arc_func)),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]); 

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
                    tokens.len() == 2
                        && matches!(tokens.get(0), Some(TokenTree::Punct(p)) if p.as_char() == '.')
                        && matches!(tokens.get(1), Some(TokenTree::Punct(p)) if p.as_char() == '.')
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
                            g.extend(vec![TokenTree::Ident(Ident::new(
                                "protocol_middlewares",
                                Span::call_site(),
                            ))]);
                            g
                        })),
                        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                    ]);

                    inheritance_block.extend(vec![TokenTree::Group(Group::new(
                        Delimiter::Brace,
                        block_content,
                    ))]);

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

            // binding.set_middlewares(middlewares);
            cont.extend(vec![
                TokenTree::Ident(Ident::new("binding", Span::call_site())),
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
        
        // Modify url_expr to inject the protocol type parameter
        let modified_url_expr = self.url_expr.expand(
            self.op.protocol.clone(), 
            self.op.fn_name.clone(), 
            Ident::new("binding", Span::call_site()),
            Ident::new("params", Span::call_site())
        ); 

        cont.extend(modified_url_expr);
        cont.extend(std::iter::once(TokenTree::Punct(Punct::new(';', Spacing::Alone)))); 
        
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

    /// Endpoint orchestrator: inner fn + wrapper fn + registration ctor.
    pub fn expand_endpoint(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        tokens.extend(self.op.generate_function());
        tokens.extend(self.op.wrapper_function());
        tokens.extend(self.reg_func());
        tokens
    }
}

