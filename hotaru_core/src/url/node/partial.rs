/// Resumable child-search state for one path segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PartialState {
    /// Search has not started for this segment yet.
    #[default]
    NotStart,
    /// A literal candidate was already tried.
    Lit,
    /// Regex candidates were searched up to the given index.
    Reg(usize),
    /// The single-segment wildcard candidate was already tried.
    Any,
    /// The catch-all wildcard candidate was already tried.
    AnyPath,
    /// No more candidates remain for this segment.
    End,
}

impl PartialState {
    /// Returns whether this state is exhausted.
    pub fn is_end(self) -> bool {
        matches!(self, Self::End)
    }
}
