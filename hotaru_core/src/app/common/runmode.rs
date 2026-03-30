/// Run mode for framework runtimes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum RunMode {
    #[default]
    Development,
    Production,
    Beta,
    Build,
}
