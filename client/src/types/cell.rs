use crate::types::ShipKind;

/// Represents a cell on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Water,
    Ship(ShipKind),
}