use thiserror::Error as ThisError;
use crate::types::board_size::BoardSize;
use crate::types::{Orientation, ShipKind};

#[derive(Clone, Debug, Eq, ThisError, PartialEq)]
pub enum GameError {
    #[error("Position [{x}, {y}] is not on the board.")]
    OutOfBoardBounds { x: usize, y: usize },

    #[error("A {ship} is not eligible for boards {size} size")]
    ShipIneligible { ship: ShipKind, size: BoardSize },

    #[error("Placing a {ship} in ({x}, {y}) in {orientation} results in out of bounds.")]
    InvalidShipPlacementBounds { ship: ShipKind, x: usize, y: usize, orientation: Orientation },

    #[error("Placing a {ship} in ({x}, {y}) in {orientation} collides in ({xc}, {yc})")]
    InvalidShipPlacementCollides {
        ship: ShipKind,
        x: usize,
        y: usize,
        orientation: Orientation,
        xc: usize,
        yc: usize,
    },
    
    #[error("You cannot place any more {kind}s for this board size.")]
    InvalidShipPlacementKind { kind: ShipKind },
}