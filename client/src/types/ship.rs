use crate::types::{Orientation, ShipKind};

/// Represents a ship placed on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ship {
    pub kind: ShipKind,
    pub x: usize,
    pub y: usize,
    pub orientation: Orientation,
}

impl Ship {
    pub fn new(kind: ShipKind, x: usize, y: usize, orientation: Orientation) -> Ship {
        Ship {
            kind,
            x,
            y,
            orientation,
        }
    }
}
