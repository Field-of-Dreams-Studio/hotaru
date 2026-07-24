use super::APHandlerDef;

/// Endpoint flavour: the DSL body becomes core's final handler.
pub(crate) struct FinalHandler {
    def: APHandlerDef,
}

impl FinalHandler {
    pub(crate) fn new(def: APHandlerDef) -> Self {
        Self { def }
    }

    pub(crate) fn def(&self) -> &APHandlerDef {
        &self.def
    }

    pub(crate) fn into_def(self) -> APHandlerDef {
        self.def
    }
}
