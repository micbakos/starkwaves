use crate::types::{Board, Orientation, Ship, ShipKind};
use crate::types::board_size::{BoardSize, SmallerBoardSize};
use rand::Rng;
use crate::prover::Prover;

mod types;
mod prover;
mod cairo;

fn main() {
    let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
    let ships = vec![
        Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal),
        Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical),
    ];

    for ship in ships.clone() {
        board.place_ship(ship)
            .expect("Destroyer was not placed");
    }

    let board_array = board.to_array();
    println!("{}", board);
    println!("\nBoard as Vec<u8> (length: {}):", board_array.len());
    println!("{:?}", board_array);

    // Generate a random salt
    let mut rng = rand::thread_rng();
    let salt: u64 = rng.gen();

    // Create the embedded prover
    let prover = Prover::new();
    match prover.validate_and_commit(&ships, BoardSize::Smaller(SmallerBoardSize::SixBySix), salt) {
        Ok(commitment) => {
            println!("\n✓ Validation successful!");
            println!("Commitment: 0x{:064x}", commitment);
        }
        Err(e) => {
            eprintln!("\n✗ Validation failed: \n{}", e);
            std::process::exit(1);
        }
    }
}
