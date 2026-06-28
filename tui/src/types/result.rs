use crate::types::error::TuiError;

pub type Result<T, E = TuiError> = std::result::Result<T, E>;