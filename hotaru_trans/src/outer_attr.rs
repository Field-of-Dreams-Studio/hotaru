use std::iter::Peekable;

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::helper::generate_compile_error;

pub struct OuterAttr {
    pub attrs: Vec<TokenStream>,
}

impl OuterAttr {
    pub fn new(attrs: Vec<TokenStream>) -> Self {
        Self { attrs }
    }

    pub fn is_empty(&self) -> bool {
        self.attrs.is_empty()
    }

    /// Get the attribute with the given name, if it exists.
    pub fn get_attr(&self, name: &str) -> Option<&TokenStream> {
        for attr in &self.attrs {
            let mut tokens = attr.clone().into_iter().peekable();
            if let Some(TokenTree::Ident(ident)) = tokens.peek() {
                if ident.to_string() == name {
                    return Some(attr);
                }
            }
        }
        None
    }

    /// Remove the first attribute with the given name and return its inner tokens.
    pub fn remove(&mut self, name: &str) -> Option<TokenStream> {
        if let Some(index) = self.attrs.iter().position(|attr| {
            let mut tokens = attr.clone().into_iter().peekable();
            match tokens.peek() {
                Some(TokenTree::Ident(ident)) => ident.to_string() == name,
                _ => false,
            }
        }) {
            Some(self.attrs.remove(index))
        } else {
            None
        }
    } 

    pub fn get_inners<T: AsRef<str>>(
        tokens: TokenStream, 
        error: T
    ) -> Result<TokenStream, TokenStream> {
        let mut iter = tokens.into_iter().peekable();
        iter.next(); // consume ident 
        match iter.next() {
            Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Parenthesis => {
                Ok(g.stream())
            }
            Some(tt) => Err(generate_compile_error(
                tt.span(),
                error.as_ref(), 
            )),
            None => Err(generate_compile_error(
                Span::call_site(),
                error.as_ref(),
            )),
        }
    }

    /// Reform the outer attributes back into a TokenStream.
    pub fn reform(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        for attr in &self.attrs {
            let mut attr_group = Group::new(Delimiter::Bracket, attr.clone());
            attr_group.set_span(Span::call_site());
            let mut hash_punct = Punct::new('#', Spacing::Alone);
            hash_punct.set_span(Span::call_site());
            tokens.extend(vec![
                TokenTree::Punct(hash_punct),
                TokenTree::Group(attr_group),
            ]);
        }
        tokens
    } 

    /// FIXME : Reform an inner attribute (e.g., `#![attr]`) into a TokenStream. 
    /// Check whetger this is correct 
    fn reform_inner_attr(
        &self, 
        name: Ident, 
        value: TokenStream 
    ) -> TokenStream {
        let mut tokens = TokenStream::new();

        let mut attr_group = Group::new(Delimiter::Bracket, {
            let mut inner_tokens = TokenStream::new();
            let mut bang_punct = Punct::new('!', Spacing::Alone);
            bang_punct.set_span(Span::call_site());
            inner_tokens.extend(vec![
                TokenTree::Ident(name),
                TokenTree::Punct(bang_punct),
                TokenTree::Group(Group::new(Delimiter::Parenthesis, value)),
            ]);
            inner_tokens
        });
        attr_group.set_span(Span::call_site());
        let mut hash_punct = Punct::new('#', Spacing::Alone);
        hash_punct.set_span(Span::call_site());
        tokens.extend(vec![
            TokenTree::Punct(hash_punct),
            TokenTree::Group(attr_group),
        ]);

        tokens 
    } 
}

/// Parses outer attributes (e.g., `#[attr]`) from the start of the token stream.
/// Removed attributes are returned as a vector of TokenStreams.
/// Each item in the vector represents one attribute (**EXCLUDES** the leading `#` and brackets).
/// Inner attributes (e.g., `#![attr]`) are rejected with a compile error.
pub fn parse_outer_attrs(
    tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
) -> Result<OuterAttr, TokenStream> {
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
                        // Without the leading '#' and brackets
                        attrs.push(g.stream());
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

    Ok(OuterAttr::new(attrs))
}

#[cfg(test)]
mod tests {
    use std::iter::Peekable;

    use proc_macro2::{
        Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree,
    };

    struct OuterAttrPm2 {
        attrs: Vec<TokenStream>,
    }

    impl OuterAttrPm2 {
        fn new(attrs: Vec<TokenStream>) -> Self {
            Self { attrs }
        }

        fn get_attr(&self, name: &str) -> Option<&TokenStream> {
            for attr in &self.attrs {
                let mut tokens = attr.clone().into_iter().peekable();
                if let Some(TokenTree::Ident(ident)) = tokens.peek() {
                    if ident.to_string() == name {
                        return Some(attr);
                    }
                }
            }
            None
        }

        fn remove(&mut self, name: &str) -> Option<TokenStream> {
            if let Some(index) = self.attrs.iter().position(|attr| {
                let mut tokens = attr.clone().into_iter().peekable();
                match tokens.peek() {
                    Some(TokenTree::Ident(ident)) => ident.to_string() == name,
                    _ => false,
                }
            }) {
                Some(self.attrs.remove(index))
            } else {
                None
            }
        }

        fn reform(&self) -> TokenStream {
            let mut tokens = TokenStream::new();
            for attr in &self.attrs {
                let mut attr_group = Group::new(Delimiter::Bracket, attr.clone());
                attr_group.set_span(Span::call_site());
                let mut hash_punct = Punct::new('#', Spacing::Alone);
                hash_punct.set_span(Span::call_site());
                tokens.extend(vec![
                    TokenTree::Punct(hash_punct),
                    TokenTree::Group(attr_group),
                ]);
            }
            tokens
        }
    }

    fn generate_compile_error_pm2(message: &str) -> TokenStream {
        let mut tokens = TokenStream::new();
        let ident = TokenTree::Ident(Ident::new("compile_error", Span::call_site()));
        let punct = TokenTree::Punct(Punct::new('!', Spacing::Alone));
        let message = TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            TokenStream::from(TokenTree::Literal(Literal::string(message))),
        ));
        let semi_column = TokenTree::Punct(Punct::new(';', Spacing::Alone));
        tokens.extend(vec![ident, punct, message, semi_column]);
        tokens
    }

    fn parse_outer_attrs_pm2(
        tokens: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<OuterAttrPm2, TokenStream> {
        let mut attrs = Vec::new();

        loop {
            match tokens.peek() {
                Some(TokenTree::Punct(p)) if p.as_char() == '#' => {
                    tokens.next();

                    match tokens.next() {
                        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket => {
                            let mut inside = g.stream().into_iter().peekable();
                            if let Some(TokenTree::Punct(p)) = inside.peek() {
                                if p.as_char() == '!' {
                                    return Err(generate_compile_error_pm2(
                                        "inner attributes (#![...] ) are not supported here",
                                    ));
                                }
                            }
                            attrs.push(g.stream());
                        }
                        Some(_) => {
                            return Err(generate_compile_error_pm2(
                                "expected attribute group after '#'",
                            ));
                        }
                        None => {
                            return Err(generate_compile_error_pm2(
                                "expected attribute group after '#'",
                            ));
                        }
                    }
                }
                _ => break,
            }
        }

        Ok(OuterAttrPm2::new(attrs))
    }

    fn ts(input: &str) -> TokenStream {
        input.parse::<TokenStream>().expect("token stream")
    }

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn parse_collects_and_reforms_attrs() {
        let input = ts("#[doc = \"hi\"] #[cfg(test)] fn foo() {}");
        let mut tokens = input.into_iter().peekable();
        let mut attrs = parse_outer_attrs_pm2(&mut tokens).expect("parse attrs");

        assert_eq!(attrs.attrs.len(), 2);
        assert!(attrs.get_attr("doc").is_some());
        assert!(attrs.get_attr("cfg").is_some());
        let removed = attrs.remove("cfg").expect("remove cfg");
        assert!(attrs.get_attr("cfg").is_none());
        assert_eq!(attrs.attrs.len(), 1);
        assert_eq!(compact(&removed.to_string()), compact("cfg(test)"));
        assert_eq!(
            compact(&attrs.reform().to_string()),
            compact("#[doc = \"hi\"]")
        );

        match tokens.next() {
            Some(TokenTree::Ident(ident)) => assert_eq!(ident.to_string(), "fn"),
            other => panic!("expected fn ident, got {:?}", other),
        }
    }

    #[test]
    fn reject_inner_attributes() {
        let input = ts("#![allow(dead_code)] fn foo() {}");
        let mut tokens = input.into_iter().peekable();
        let err = match parse_outer_attrs_pm2(&mut tokens) {
            Ok(_) => panic!("expected error"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains("compile_error")
                || err
                    .to_string()
                    .contains("inner attributes (#![...] ) are not supported here"),
            "unexpected error output: {}",
            err.to_string()
        );
    }

    #[test]
    fn remove_missing_attr_returns_none() {
        let input = ts("#[doc = \"hi\"] fn foo() {}");
        let mut tokens = input.into_iter().peekable();
        let mut attrs = parse_outer_attrs_pm2(&mut tokens).expect("parse attrs");
        assert!(attrs.remove("cfg").is_none());
        assert_eq!(attrs.attrs.len(), 1);
    }
}

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

