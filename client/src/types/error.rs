use crate::types::board_size::BoardSize;
use crate::types::{Orientation, ShipKind};
use thiserror::Error as ThisError;

#[derive(Clone, Debug, Eq, ThisError, PartialEq)]
pub enum GameError {
    #[error("A {ship} is not eligible for boards {size} size")]
    ShipIneligible { ship: ShipKind, size: BoardSize },

    #[error("Placing a {ship} in ({x}, {y}) in {orientation} results in out of bounds.")]
    InvalidShipPlacementBounds { ship: ShipKind, x: u8, y: u8, orientation: Orientation },

    #[error("Placing a {ship} in ({x}, {y}) in {orientation} collides in ({xc}, {yc})")]
    InvalidShipPlacementCollides {
        ship: ShipKind,
        x: u8,
        y: u8,
        orientation: Orientation,
        xc: u8,
        yc: u8,
    },

    #[error("You cannot place any more {kind}s for this board size.")]
    InvalidShipPlacementKind { kind: ShipKind },

    #[error("All ships are not placed in board")]
    BoardNotReady,
}