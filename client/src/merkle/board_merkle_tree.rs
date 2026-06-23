use crate::merkle::pedersen_hasher::PedersenHasher;
use rs_merkle::MerkleTree;
use starknet_rust::core::crypto::pedersen_hash;
use starknet_rust::core::types::Felt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardMerkleTree {
    leaves: Vec<[u8; 32]>,
    salt: u64,
}

/// Produces a Merkle Tree that uses Pedersen hash for node hashing
impl BoardMerkleTree {
    pub fn build(board_array: Vec<bool>, salt: u64) -> Self {
        let salt_felt = Felt::from(salt);

        let leaves = board_array
            .iter()
            .map(|c| {
                let cell = Felt::from(*c);
                pedersen_hash(&cell, &salt_felt).to_bytes_be()
            })
            .collect::<Vec<_>>();
        BoardMerkleTree { leaves, salt }
    }

    pub fn as_tree(&self) -> MerkleTree<PedersenHasher> {
        MerkleTree::<PedersenHasher>::from_leaves(&self.leaves)
    }

    pub fn root(&self) -> Felt {
        let tree = self.as_tree();
        Felt::from_bytes_be(&tree.root().expect("Tree should have root"))
    }

    pub fn proof(&self, offset: usize) -> Vec<Felt> {
        let tree = self.as_tree();
        let proof = tree.proof(vec![offset].as_slice());

        proof
            .to_bytes()
            .chunks(32)
            .map(|c| Felt::from_bytes_be_slice(c))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::board_size::{BoardSize, SmallerBoardSize};
    use crate::types::{Board, Orientation, Ship, ShipKind};

    #[test]
    fn test_empty_board_commitment_fails() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));

        let commit_result = board.commit(1234);

        assert!(
            commit_result.is_err(),
            "Empty board should not be ready for commitment"
        );
    }

    #[test]
    fn test_same_board_same_salt_produces_same_commitment() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        // Complete board: needs 1 Destroyer + 1 Cruiser for 6x6
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let mut board2 = board.clone();

        let salt = 12345u64;
        let root1 = board.commit(salt).unwrap();
        let root2 = board2.commit(salt).unwrap();

        assert_eq!(
            root1, root2,
            "Same board and salt should produce same commitment"
        );
    }

    #[test]
    fn test_different_salt_produces_different_commitment() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        // Complete board: needs 1 Destroyer + 1 Cruiser for 6x6
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let mut board2 = board.clone();

        let root1 = board.commit(0).unwrap();
        let root2 = board2.commit(1).unwrap();

        assert_ne!(
            root1, root2,
            "Different salts should produce different commitments"
        );
    }

    #[test]
    fn test_different_boards_produce_different_commitments() {
        // Board 1: Complete board with ships at position 1
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board1
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Board 2: Complete board with ships at position 2
        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2
            .place_ship(Ship::new(ShipKind::Cruiser, 1, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");
        board2
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                3,
                3,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");

        let salt = 12345u64;
        let root1 = board1.commit(salt).unwrap();
        let root2 = board2.commit(salt).unwrap();

        assert_ne!(
            root1, root2,
            "Different boards should produce different commitments"
        );
    }

    #[test]
    fn test_ship_position_affects_commitment() {
        let salt = 12345u64;

        // Board 1: Complete board with ships at position 1
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board1
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Board 2: Complete board with ships at position 2
        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                3,
                3,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board2
            .place_ship(Ship::new(ShipKind::Cruiser, 0, 4, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let root1 = board1.commit(salt).unwrap();
        let root2 = board2.commit(salt).unwrap();

        assert_ne!(
            root1, root2,
            "Different ship positions should produce different commitments"
        );
    }

    #[test]
    fn test_ship_orientation_affects_commitment() {
        let salt = 12345u64;

        // Board 1: Complete board with Cruiser horizontal
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1
            .place_ship(Ship::new(ShipKind::Cruiser, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");
        board1
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                2,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");

        // Board 2: Complete board with Cruiser vertical
        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2
            .place_ship(Ship::new(ShipKind::Cruiser, 0, 0, Orientation::Vertical))
            .expect("Ship placement should succeed");
        board2
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                3,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");

        let root1 = board1.commit(salt).unwrap();
        let root2 = board2.commit(salt).unwrap();

        assert_ne!(
            root1, root2,
            "Different ship orientations should produce different commitments"
        );
    }

    #[test]
    fn test_commitment_with_multiple_ships() {
        let salt = 12345u64;

        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Destroyer placement should succeed");
        board
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Cruiser placement should succeed");

        let mut copied_board = board.clone();

        let root1 = board.commit(salt).unwrap();
        assert_ne!(
            root1,
            Felt::ZERO,
            "Root with multiple ships should not be zero"
        );

        // Verify determinism
        let root2 = copied_board.commit(salt).unwrap();
        assert_eq!(root1, root2, "Multiple builds should produce same root");
    }

    #[test]
    fn test_board_size_affects_commitment() {
        let salt = 12345u64;

        // Complete 6x6 board
        let mut board6x6 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board6x6
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board6x6
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Complete 8x8 board
        let mut board8x8 = Board::new(BoardSize::Smaller(SmallerBoardSize::EightByEight));
        board8x8
            .place_ship(Ship::new(
                ShipKind::Destroyer,
                0,
                0,
                Orientation::Horizontal,
            ))
            .expect("Ship placement should succeed");
        board8x8
            .place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let root6x6 = board6x6.commit(salt).unwrap();
        let root8x8 = board8x8.commit(salt).unwrap();

        assert_ne!(
            root6x6, root8x8,
            "Different board sizes should produce different commitments"
        );
    }
}
