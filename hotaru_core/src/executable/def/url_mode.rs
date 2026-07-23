/// Whether the raw `url` string is parsed with the protocol's pattern
/// tokenizer or with the legacy `/`-splitting literal grammar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UrlMode {
    #[default]
    Pattern,
    Literal,
}
