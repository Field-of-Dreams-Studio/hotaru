use proc_macro::{Ident, TokenStream};

use crate::outer_attr::OuterAttr;

/// Common syntax captured from an endpoint or outpoint body declaration.
pub(crate) struct APHandlerDef {
    attrs: OuterAttr,
    is_pub: bool,
    protocol: Ident,
    request: Ident,
    body: TokenStream,
}

impl APHandlerDef {
    pub(crate) fn new(
        attrs: OuterAttr,
        is_pub: bool,
        protocol: Ident,
        request: Ident,
        body: TokenStream,
    ) -> Self {
        Self {
            attrs,
            is_pub,
            protocol,
            request,
            body,
        }
    }

    pub(crate) fn attrs(&self) -> &OuterAttr {
        &self.attrs
    }

    pub(crate) fn is_pub(&self) -> bool {
        self.is_pub
    }

    pub(crate) fn protocol(&self) -> &Ident {
        &self.protocol
    }

    pub(crate) fn request(&self) -> &Ident {
        &self.request
    }

    pub(crate) fn body(&self) -> &TokenStream {
        &self.body
    }

    pub(crate) fn into_parts(self) -> (OuterAttr, bool, Ident, Ident, TokenStream) {
        (
            self.attrs,
            self.is_pub,
            self.protocol,
            self.request,
            self.body,
        )
    }
}
