use crate::types::{Orientation, ShipKind};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a ship placed on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ship {
    pub id: Uuid,
    pub kind: ShipKind,
    pub x: u8,
    pub y: u8,
    pub orientation: Orientation,
}

impl Ship {
    pub fn new(kind: ShipKind, x: u8, y: u8, orientation: Orientation) -> Ship {
        Ship {
            id: Uuid::new_v4(),
            kind,
            x,
            y,
            orientation,
        }
    }
}
