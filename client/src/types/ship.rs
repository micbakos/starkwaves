use crate::types::{Orientation, ShipKind};
use serde::{Deserialize, Serialize};

/// Represents a ship placed on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ship {
    pub kind: ShipKind,
    pub x: u8,
    pub y: u8,
    pub orientation: Orientation,
}

impl Ship {
    pub fn new(kind: ShipKind, x: u8, y: u8, orientation: Orientation) -> Ship {
        Ship {
            kind,
            x,
            y,
            orientation,
        }
    }
}