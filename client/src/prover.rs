use crate::cairo::cairo_runner::CairoRunner;
use crate::cairo::cairo_value::CairoValue;
use crate::cairo::panic_result::CairoError;
use crate::types::board_size::BoardSize;
use crate::types::error::GameError;
use crate::types::result::Result;
use crate::types::{Orientation, Ship};
use cairo_native::Value;
use starknet::core::types::Felt;

pub struct Prover {
    runner: CairoRunner,
}

impl Prover {
    /// Create a new prover by loading and compiling the Sierra program
    pub fn new() -> Self {
        let runner = CairoRunner::new();

        Self { runner }
    }

    /// Validate ship placement and create a commitment
    /// Returns the commitment as a Felt (felt252)
    pub fn validate_and_commit(
        &self,
        ships: &[Ship],
        board_size: BoardSize,
        salt: u64,
    ) -> Result<Felt> {
        let function_name = "starkwaves_validator::validate_and_commit";

        let ships_array = ships
            .iter()
            .map(|ship| {
                let cairo_value = Into::<CairoValue>::into(*ship);
                cairo_value.0
            })
            .collect::<Vec<Value>>();

        let board_size_value: CairoValue = Into::into(board_size);

        let args = vec![
            Value::Array(ships_array),
            board_size_value.0,
            Value::Felt252(Felt::from(salt)),
        ];

        self.runner
            .execute_cairo_fn(&function_name, args)
            .and_then(|value| {
                if let Value::Felt252(felt) = value {
                    Ok(felt)
                } else {
                    Err(CairoError::from_values(vec![value], "Expected Felt252 as return type of validate_and_commit"))
                }
            })
            .map_err(|err| GameError::ProverError { cairo_error: err })
    }
}

#[cfg(test)]
mod tests {
    use crate::types::board_size::SmallerBoardSize;
    use super::*;
    use crate::types::ShipKind;

    #[test]
    fn test_prover_validates_correct_placement() {
        let prover = Prover::new();

        let ships = vec![
            Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal),
            Ship::new(ShipKind::Cruiser, 2, 2, Orientation::Vertical),
        ];

        let commitment = prover
            .validate_and_commit(&ships, BoardSize::Smaller(SmallerBoardSize::SixBySix), 12345)
            .expect("Validation should succeed");

        println!("Commitment: {:?}", commitment);
        assert_ne!(commitment, Felt::ZERO);
    }

    #[test]
    fn test_prover_rejects_overlapping_ships() {
        let prover = Prover::new();

        // These ships overlap at (0,0)
        let ships = vec![
            Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal),
            Ship::new(ShipKind::Cruiser, 0, 0, Orientation::Vertical),
        ];

        let result = prover.validate_and_commit(&ships, BoardSize::Smaller(SmallerBoardSize::SixBySix), 12345);
        assert!(result.is_err(), "Should reject overlapping ships");
    }
}
