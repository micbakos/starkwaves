use starkwaves_client::types::board_size::{BoardSize, SmallerBoardSize};
use starkwaves_client::types::{Board, Orientation, Ship, ShipKind};
use rand::Rng;

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

    let mut rng = rand::thread_rng();
    let salt: u64 = rng.gen();

    let board_array = board.to_array();
    println!("{}", board);
    println!("\nBoard as Vec<u8> (length: {}):", board_array.len());
    println!("{:?}", board_array);

    let commitment = board.commitment(salt);
    println!("\n✓ Commitment generated (Rust Poseidon)");
    println!("Commitment: 0x{:064x}", commitment);
    println!("Salt: {}", salt);
}
