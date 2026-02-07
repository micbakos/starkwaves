pub mod board;
pub mod fire;
pub mod outcome;
pub mod ship;

pub use board::{
    AllBoardSizesTrait, BoardSize, BoardSizeTrait, LargerBoardSize, SmallerBoardSize, create_board,
};
pub use fire::{FireStatus, FireStatusTrait, HitReport};
pub use outcome::{Outcome, OutcomeBeforeReveal, OutcomeBeforeRevealTrait, RevealStatus};
pub use ship::{Orientation, Ship, ShipKind, ShipKindTrait};
