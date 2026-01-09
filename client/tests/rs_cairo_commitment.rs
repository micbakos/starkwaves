use crate::cairo::cairo_runner::CairoRunner;
use cairo_native::Value;
use starknet::core::types::Felt;
use starkwaves_client::types::Board;
use std::path::Path;
use crate::cairo::panic_result::CairoError;

pub struct CairoBoardCommitmentRunner {
    runner: CairoRunner,
}

impl CairoBoardCommitmentRunner {
    pub fn new() -> CairoBoardCommitmentRunner {
        let sierra_path = env!("VALIDATOR_SIERRA_PATH");
        println!("Loading Sierra program from {}...", sierra_path);

        let runner = CairoRunner::new(Path::new(sierra_path));
        CairoBoardCommitmentRunner { runner }
    }

    pub fn for_board(&self, board: &Board, salt: u64) -> Felt {
        let board_items = board
            .to_array()
            .iter()
            .map(|i| Value::Felt252(Felt::from(*i)))
            .collect::<Vec<_>>();

        let board_array = Value::Array(board_items);
        let salt_value = Value::Felt252(Felt::from(salt));

        self.runner
            .execute_cairo_fn(
                "starkwaves_validator::compute_merkle_root",
                vec![board_array, salt_value],
            )
            .and_then(|value| {
                if let Value::Felt252(felt) = value {
                    Ok(felt)
                } else {
                    Err(CairoError::from_values(
                        vec![value],
                        "Expected Felt252 as return type of compute_merkle_root",
                    ))
                }
            }).expect("Failed to compute merkle root")
    }
}

#[cfg(test)]
mod tests {
    use starkwaves_client::types::board_size::{BoardSize, SmallerBoardSize};
    use starkwaves_client::types::{Orientation, Ship, ShipKind};
    use super::*;

    fn rs_commitment(ships: Vec<Ship>, board_size: BoardSize, salt: u64) -> (Board, Felt) {
        let mut board = Board::new(board_size);
        for ship in ships.clone() {
            board.place_ship(ship)
                .expect("Ship placement failed");
        }

        (board.clone(), board.commitment(salt))
    }

    fn cairo_commitment(board: &Board, salt: u64) -> Felt {
        let cairo_commitment_runner = CairoBoardCommitmentRunner::new();
        cairo_commitment_runner.for_board(board, salt)
    }

    #[test]
    fn test_commitment() {
        let salt = 1234;
        let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
        let ships = vec![
            Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal),
            Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical),
        ];

        let (board, rs_commitment) = rs_commitment(ships.clone(), board_size, salt);
        let cairo_commitment = cairo_commitment(&board, salt);

        assert_eq!(rs_commitment, cairo_commitment);
    }
}
