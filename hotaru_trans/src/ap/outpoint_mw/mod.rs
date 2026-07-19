mod send;

use proc_macro::TokenStream;

use super::APHandlerDef;

/// Outpoint flavour: the DSL body becomes middleware before core's sender.
pub(crate) struct OutpointMW {
    def: APHandlerDef,
}

impl OutpointMW {
    pub(crate) fn new(def: APHandlerDef) -> Self {
        Self { def }
    }

    pub(crate) fn def(&self) -> &APHandlerDef {
        &self.def
    }

    pub(crate) fn rewritten_body(&self) -> TokenStream {
        send::rewrite_send(self.def.body().clone(), self.def.request())
    }

    pub(crate) fn into_def(self) -> APHandlerDef {
        self.def
    }
}
