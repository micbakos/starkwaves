use crate::types::{Board, Orientation, Ship, ShipKind};
use crate::types::board_size::{BoardSize, LargerBoardSize};

mod types;

fn main() {
    let mut board = Board::default();

    board.place_ship(Ship::new(ShipKind::Cruiser, 0, 0, Orientation::Horizontal))
        .expect("Cruiser was not placed");
    board.place_ship(Ship::new(ShipKind::Destroyer, 1, 0, Orientation::Vertical))
        .expect("Destroyer was not placed");

    println!("{}", board);
}
