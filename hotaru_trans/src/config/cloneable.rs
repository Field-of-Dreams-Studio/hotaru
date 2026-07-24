/// Selects the runtime parameter container produced for this configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cloneable {
    /// Configuration values must support `Clone` (`ParamsClone`).
    Yes,
    /// Configuration values need not support `Clone` (`Params`).
    No,
}

impl Cloneable {
    pub(crate) fn is_cloneable(self) -> bool {
        matches!(self, Self::Yes)
    }
}
