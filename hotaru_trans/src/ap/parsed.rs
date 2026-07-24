use super::{APHandlerDef, APParts};

/// Fully parsed components shared before endpoint/outpoint wrapping.
pub(crate) struct ParsedAP {
    parts: APParts,
    handler: APHandlerDef,
}

impl ParsedAP {
    pub(crate) fn new(parts: APParts, handler: APHandlerDef) -> Self {
        Self { parts, handler }
    }

    pub(crate) fn into_parts(self) -> (APParts, APHandlerDef) {
        (self.parts, self.handler)
    }
}
