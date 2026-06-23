use crate::types::error::GameError;
use derive_more::Display;
use serde::{Deserialize, Serialize};

/// Represents the orientation of a ship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Orientation {
    #[display("horizontal")]
    Horizontal,
    #[display("vertical")]
    Vertical,
}

impl TryFrom<&str> for Orientation {
    type Error = GameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "h" => Ok(Orientation::Horizontal),
            "v" => Ok(Orientation::Vertical),
            _ => Err(GameError::InvalidInput {
                expected: "h|v".to_string(),
                received: value.to_string(),
            }),
        }
    }
}
