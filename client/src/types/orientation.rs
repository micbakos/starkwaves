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