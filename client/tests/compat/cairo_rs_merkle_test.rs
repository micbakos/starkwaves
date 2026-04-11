#[cfg(test)]
mod tests {
    use crate::compat::cairo::cairo_runner::CairoRunner;
    use cairo_native::Value;
    use starknet_rust::core::crypto::pedersen_hash;
    use starknet_rust::core::types::Felt;
    use starkwaves_client::types::board_size::{BoardSize, SmallerBoardSize};
    use starkwaves_client::types::fire_report::FireStatus;
    use starkwaves_client::types::Board;
    use starkwaves_client::types::{Orientation, Ship, ShipKind};
    use std::path::Path;
    use crate::compat::cairo::panic_result::CairoError;

    pub struct CairoMerkleRunner {
        runner: CairoRunner,
    }

    impl CairoMerkleRunner {
        pub fn new() -> CairoMerkleRunner {
            let sierra_path = env!("MERKLE_SIERRA_PATH");
            let runner = CairoRunner::new(Path::new(sierra_path));
            CairoMerkleRunner { runner }
        }

        pub fn verify(
            &self,
            salted_status: Felt,
            proof: Vec<Felt>,
            root: Felt,
            index: usize
        ) -> bool {
            self.runner
                .execute_cairo_fn(
                    "merkle::verify",
                    vec![
                        Value::Felt252(salted_status),
                        Value::Array(proof.iter().map(|p| Value::Felt252(*p)).collect()),
                        Value::Felt252(root),
                        Value::Uint32(index as u32),
                    ],
                )
                .and_then(|value: Value| {
                    if let Value::Enum {
                        tag, debug_name, ..
                    } = value.clone()
                    {
                        if let Some(name) = debug_name
                            && name == "core::bool"
                        {
                            Ok(tag == 1)
                        } else {
                            Err(CairoError::from_values(
                                vec![value],
                                "Expected bool enum as return type of verify_report",
                            ))
                        }
                    } else {
                        Err(CairoError::from_values(
                            vec![value],
                            "Expected bool enum as return type of verify_report",
                        ))
                    }
                })
                .expect("Failed to verify")
        }
    }

    #[test]
    fn verify_hit_in_cairo() {
        let cairo_merkle_runner = CairoMerkleRunner::new();

        let salt = 1234;
        let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
        let ships = vec![
            Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal),
            Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical),
        ];

        let mut board = Board::new(board_size);
        for ship in ships.clone() {
            board.place_ship(ship).expect("Ship placement failed");
        }
        let root = board.commit(salt).expect("Should commit");

        let report = board.receive_fire(0, 0).unwrap();
        assert_eq!(report.status, FireStatus::Hit(ShipKind::Destroyer));
        assert!(report.ship_destroyed.is_none());
        assert!(!report.proof.is_empty());

        let cairo_verified = cairo_merkle_runner.verify(
            report.salted_status_value(salt),
            report.proof.clone(),
            root,
            0
        );

        assert!(cairo_verified);
    }

    #[test]
    fn fake_miss_is_not_verified_in_cairo() {
        let cairo_merkle_runner = CairoMerkleRunner::new();

        let salt = 1234;
        let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
        let ships = vec![
            Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal),
            Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical),
        ];

        let mut board = Board::new(board_size);
        for ship in ships.clone() {
            board.place_ship(ship).expect("Ship placement failed");
        }
        let root = board.commit(salt).expect("Should commit");
        let report = board.receive_fire(0, 0).unwrap();

        // Get the proof form a real successful shot
        let proof = report.proof;
        // let's assume that client claims that 0x0 is a miss
        // so let's hash water with salt (what an irony...)
        let fake_salted_status = pedersen_hash(&Felt::from(0), &Felt::from(salt));

        let cairo_verified = cairo_merkle_runner.verify(
            fake_salted_status,
            proof,
            root,
            0
        );

        assert!(!cairo_verified);
    }
}
