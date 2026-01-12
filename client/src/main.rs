use starkwaves_client::types::board_size::{BoardSize, SmallerBoardSize};
use starkwaves_client::types::{Board, Orientation, Ship, ShipKind};

fn main() {
    let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
    let ships = vec![
        Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal),
        Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical),
    ];

    for ship in ships.clone() {
        board.place_ship(ship)
            .expect("Ship placement failed");
    }

    let salt = 1234;

    let commitment = board.commit(salt).expect("Board should be ready");
    println!("{}", board);
    println!("Salt: {}", salt);
    println!("Commitment: 0x{:064x}", commitment);
}
