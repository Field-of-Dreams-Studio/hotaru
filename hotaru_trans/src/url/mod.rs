pub(self) mod urlargs;
pub(self) mod urlexpr;
pub(self) mod url_func;
pub(self) mod send;

pub(crate) mod parse;

pub(crate) mod endpoint;
pub(crate) mod outpoint;

pub(crate) use endpoint::{endpoint_trans, endpoint_attr, endpoint_semi_trans};
pub(crate) use outpoint::{outpoint_trans, outpoint_attr, outpoint_semi_trans};
