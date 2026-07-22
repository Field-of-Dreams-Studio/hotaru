use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::helper::use_core;
use crate::outer_attr::OuterAttr;

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
        // Helper: core/alloc path-like builder
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
                ts.extend(core::iter::once(TokenTree::Ident(Ident::new(
                    p,
                    Span::call_site(),
                ))));
                first = false;
            }
            ts
        }

        // #[...] [pub] struct <Name>;
        let mut struct_decl = TokenStream::new();
        struct_decl.extend(self.attrs.reform());

        // #[allow(non_camel_case_types)]
        let mut inner = TokenStream::new();
        inner.extend(vec![
            TokenTree::Ident(Ident::new("allow", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, {
                let mut g = TokenStream::new();
                g.extend(core::iter::once(TokenTree::Ident(Ident::new(
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

        // Generated outpoint middleware stays private.
        if self.is_pub {
            struct_decl.extend(core::iter::once(TokenTree::Ident(Ident::new(
                "pub",
                Span::call_site(),
            ))));
        }
        struct_decl.extend(vec![
            TokenTree::Ident(Ident::new("struct", Span::call_site())),
            TokenTree::Ident(self.name.clone()),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);

        // Use a type alias to avoid the << ambiguity at the start of generic parameters.
        // Instead of: impl AsyncMiddleware<<Protocol as Protocol>::Context> for Name
        // We generate: type _Ctx = <Protocol as Protocol>::Context;
        //              impl AsyncMiddleware<_Ctx> for Name
        let mut type_alias = TokenStream::new();
        type_alias.extend(vec![
            TokenTree::Ident(Ident::new("type", Span::call_site())),
            TokenTree::Ident(Ident::new("_Ctx", Span::call_site())),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
            // <Protocol as Protocol>::Context
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Ident(self.protocol.clone()),
            TokenTree::Ident(Ident::new("as", Span::call_site())),
            TokenTree::Group(Group::new(
                Delimiter::None,
                use_core(&["protocol", "Protocol"]),
            )),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new("Context", Span::call_site())),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);

        // impl AsyncMiddleware<_Ctx> for Name
        let mut impl_head = TokenStream::new();
        impl_head.extend(vec![
            TokenTree::Ident(Ident::new("impl", Span::call_site())),
            TokenTree::Group(Group::new(
                Delimiter::None,
                use_core(&["executable", "middleware", "AsyncMiddleware"]),
            )),
            TokenTree::Punct(Punct::new('<', Spacing::Alone)),
            TokenTree::Ident(Ident::new("_Ctx", Span::call_site())),
            TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            TokenTree::Ident(Ident::new("for", Span::call_site())),
            TokenTree::Ident(self.name.clone()),
        ]);

        // fn as_any(&self) -> &dyn core::any::Any { self }
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
            let any_path = path(&["core", "any", "Any"]);
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

        // Use core's aliases as the middleware ABI source of truth. They
        // select the correct `spawn_send` or `spawn_local` trait objects.
        let mut handle_fn = TokenStream::new();
        {
            // &'a self, context: _Ctx, next: Box<NextFn<_Ctx>>
            let mut params = TokenStream::new();
            params.extend(vec![
                TokenTree::Punct(Punct::new('&', Spacing::Alone)),
                TokenTree::Punct(Punct::new('\'', Spacing::Joint)),
                TokenTree::Ident(Ident::new("a", Span::call_site())),
                TokenTree::Ident(Ident::new("self", Span::call_site())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            ]);
            params.extend(vec![
                TokenTree::Ident(Ident::new("context", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
                TokenTree::Ident(Ident::new("_Ctx", Span::call_site())),
                TokenTree::Punct(Punct::new(',', Spacing::Alone)),
            ]);
            let mut next_ty = TokenStream::new();
            next_ty.extend(use_core(&["prelude", "Box"]));
            next_ty.extend(vec![
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                TokenTree::Group(Group::new(
                    Delimiter::None,
                    use_core(&["executable", "middleware", "NextFn"]),
                )),
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                TokenTree::Ident(Ident::new("_Ctx", Span::call_site())),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            ]);
            params.extend(vec![
                TokenTree::Ident(Ident::new("next", Span::call_site())),
                TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            ]);
            params.extend(next_ty);

            // -> BoxFuture<_Ctx>
            let mut ret_ty = TokenStream::new();
            ret_ty.extend(use_core(&["executable", "middleware", "BoxFuture"]));
            ret_ty.extend(vec![
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                TokenTree::Ident(Ident::new("_Ctx", Span::call_site())),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            ]);

            // Box::pin(async move { let mut <req> = context; <user body> })
            let mut body = TokenStream::new();
            body.extend(use_core(&["prelude", "Box", "pin"]));
            body.extend(core::iter::once(TokenTree::Group(Group::new(
                Delimiter::Parenthesis,
                {
                    let mut async_move = TokenStream::new();
                    async_move.extend(vec![
                        TokenTree::Ident(Ident::new("async", Span::call_site())),
                        TokenTree::Ident(Ident::new("move", Span::call_site())),
                    ]);
                    let mut block = TokenStream::new();
                    block.extend(vec![
                        TokenTree::Ident(Ident::new("let", Span::call_site())),
                        TokenTree::Ident(Ident::new("mut", Span::call_site())),
                        TokenTree::Ident(self.req_var_name.clone()),
                        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
                        TokenTree::Ident(Ident::new("context", Span::call_site())),
                        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
                    ]);
                    block.extend(self.cont.clone());
                    async_move.extend(core::iter::once(TokenTree::Group(Group::new(
                        Delimiter::Brace,
                        block,
                    ))));
                    async_move
                },
            ))));

            handle_fn.extend(vec![
                TokenTree::Ident(Ident::new("fn", Span::call_site())),
                TokenTree::Ident(Ident::new("handle", Span::call_site())),
                TokenTree::Punct(Punct::new('<', Spacing::Alone)),
                TokenTree::Punct(Punct::new('\'', Spacing::Joint)),
                TokenTree::Ident(Ident::new("a", Span::call_site())),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, params)),
                TokenTree::Punct(Punct::new('-', Spacing::Joint)),
                TokenTree::Punct(Punct::new('>', Spacing::Alone)),
            ]);
            handle_fn.extend(ret_ty);
            handle_fn.extend(vec![TokenTree::Group(Group::new(Delimiter::Brace, body))]);
        }

        // impl body { as_any .. return_self .. handle .. }
        let mut impl_body = TokenStream::new();
        impl_body.extend(as_any_fn);
        impl_body.extend(return_self_fn);
        // dbg!(handle_fn.clone().to_string());
        impl_body.extend(handle_fn);

        // Final assembly.
        //
        // The struct stays at module scope (users name it directly). The
        // `type _Ctx = ...` alias is wrapped in an anonymous `const _: () = { ... };`
        // scope alongside the impl so the alias doesn't leak into the
        // surrounding module — otherwise two middlewares in the same module
        // would each define `_Ctx` at module scope and collide.
        let mut impl_block = TokenStream::new();
        impl_block.extend(impl_head);
        impl_block.extend(core::iter::once(TokenTree::Group(Group::new(
            Delimiter::Brace,
            impl_body,
        ))));

        let mut const_scope_body = TokenStream::new();
        const_scope_body.extend(type_alias);
        const_scope_body.extend(impl_block);

        let mut out = TokenStream::new();
        out.extend(struct_decl);
        out.extend(vec![
            TokenTree::Ident(Ident::new("const", Span::call_site())),
            TokenTree::Ident(Ident::new("_", Span::call_site())),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
            TokenTree::Punct(Punct::new('=', Spacing::Alone)),
            TokenTree::Group(Group::new(Delimiter::Brace, const_scope_body)),
            TokenTree::Punct(Punct::new(';', Spacing::Alone)),
        ]);
        out
    }
}
