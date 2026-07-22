use crate::app::{
    AppTarget,
    common::{OperationalConfig, RunMode},
};
use crate::connection::TransportSpec;

use super::Blueprint;

/// A Blueprint plus construction defaults.
pub struct ConfiguredBlueprint<TS: TransportSpec, T: AppTarget> {
    pub(crate) blueprint: Blueprint<TS, T>,
    pub(crate) mode: Option<RunMode>,
    pub(crate) operational: Option<OperationalConfig>,
}
