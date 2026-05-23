use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::OuterAttr;
use crate::middleware::MWFunc;
use crate::url::send::rewrite_send;

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

    pub(crate) fn wrapper_function(&self) -> TokenStream {
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
            TokenTree::Ident(Ident::new("Ok", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                let mut g = TokenStream::new();
                g.extend(vec![TokenTree::Ident(self.req_var_name.clone())]);
                g
            })),
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
            // Result<<Protocol as Protocol>::Context, <<Protocol as Protocol>::Context as RequestContext>::Error>
            TokenTree::Ident(Ident::new("Result", Span::call_site())),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
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
            // <<Protocol as Protocol>::Context as RequestContext>::Error
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
            TokenTree::Ident(Ident::new("Error", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Brace, cont)),
        ]);
        tokens
    }

    /// Expand the outpoint body into a `__Outpoint_MW_<fn_name>` struct
    /// plus its `AsyncMiddleware` trait impl (via [`MWFunc::expand`]).
    pub fn expand_middleware(&self) -> TokenStream {
        // Rewrite `send;` -> `<req> = next(<req>).await?;`
        let body = rewrite_send(self.fn_cont.clone(), &self.req_var_name);

        // Build an MWFunc with the synthetic name.
        let mw_name = Ident::new(
            &format!("__Outpoint_MW_{}", self.fn_name),
            Span::call_site(),
        );
        let mw = MWFunc::new(
            false,
            mw_name,
            self.protocol.clone(),
            self.req_var_name.clone(),
            body,
            self.attrs.clone(),
        );

        // Reuse the middleware codegen.
        mw.expand()
    }
}
