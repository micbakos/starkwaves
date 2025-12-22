use crate::types::error::GameError;

pub type Result<T, E = GameError> = std::result::Result<T, E>;