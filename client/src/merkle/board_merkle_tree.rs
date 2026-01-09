use crate::merkle::pedersen_hasher::PedersenHasher;
use rs_merkle::MerkleTree;
use starknet::core::crypto::pedersen_hash;
use starknet::core::types::Felt;

pub struct BoardMerkleTree {
    tree: MerkleTree<PedersenHasher>,
}

/// Produces a Merkle Tree that uses Pedersen hash for node hashing
impl BoardMerkleTree {
    pub fn build(board_array: Vec<u8>, salt: u64) -> Self {
        let salt = Felt::from(salt);

        let leaves = board_array
            .iter()
            .map(|c| {
                let cell = Felt::from(*c);
                let hash = pedersen_hash(&cell, &salt);
                hash.to_bytes_be()
            })
            .collect::<Vec<_>>();

        let tree = MerkleTree::<PedersenHasher>::from_leaves(&leaves);
        BoardMerkleTree { tree }
    }

    pub fn root(&self) -> Felt {
        Felt::from_bytes_be(&self.tree.root().expect("Tree should have root"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::board_size::{BoardSize, SmallerBoardSize};
    use crate::types::{Board, Orientation, Ship, ShipKind};

    #[test]
    fn test_empty_board_commitment_fails() {
        let board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));

        let result = board.to_array();

        assert!(result.is_err(), "Empty board should not be ready for commitment");
    }

    #[test]
    fn test_same_board_same_salt_produces_same_commitment() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        // Complete board: needs 1 Destroyer + 1 Cruiser for 6x6
        board.place_ship(Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");
        board.place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let salt = 12345u64;
        let board_array = board.to_array().expect("Board should be ready");

        let tree1 = BoardMerkleTree::build(board_array.clone(), salt);
        let tree2 = BoardMerkleTree::build(board_array, salt);

        assert_eq!(tree1.root(), tree2.root(), "Same board and salt should produce same commitment");
    }

    #[test]
    fn test_different_salt_produces_different_commitment() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        // Complete board: needs 1 Destroyer + 1 Cruiser for 6x6
        board.place_ship(Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");
        board.place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let board_array = board.to_array().expect("Board should be ready");

        let tree1 = BoardMerkleTree::build(board_array.clone(), 11111);
        let tree2 = BoardMerkleTree::build(board_array, 22222);

        assert_ne!(tree1.root(), tree2.root(), "Different salts should produce different commitments");
    }

    #[test]
    fn test_different_boards_produce_different_commitments() {
        // Board 1: Complete board with ships at position 1
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1.place_ship(Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");
        board1.place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Board 2: Complete board with ships at position 2
        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2.place_ship(Ship::new(ShipKind::Cruiser, 1, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");
        board2.place_ship(Ship::new(ShipKind::Destroyer, 3, 3, Orientation::Horizontal))
            .expect("Ship placement should succeed");

        let salt = 12345u64;

        let board1_array = board1.to_array().expect("Board should be ready");
        let board2_array = board2.to_array().expect("Board should be ready");

        let tree1 = BoardMerkleTree::build(board1_array, salt);
        let tree2 = BoardMerkleTree::build(board2_array, salt);

        assert_ne!(tree1.root(), tree2.root(), "Different boards should produce different commitments");
    }

    #[test]
    fn test_ship_position_affects_commitment() {
        let salt = 12345u64;

        // Board 1: Complete board with ships at position 1
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1.place_ship(Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");
        board1.place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Board 2: Complete board with ships at position 2
        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2.place_ship(Ship::new(ShipKind::Destroyer, 3, 3, Orientation::Horizontal))
            .expect("Ship placement should succeed");
        board2.place_ship(Ship::new(ShipKind::Cruiser, 0, 4, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let board1_array = board1.to_array().expect("Board should be ready");
        let board2_array = board2.to_array().expect("Board should be ready");

        let tree1 = BoardMerkleTree::build(board1_array, salt);
        let tree2 = BoardMerkleTree::build(board2_array, salt);

        assert_ne!(tree1.root(), tree2.root(), "Different ship positions should produce different commitments");
    }

    #[test]
    fn test_ship_orientation_affects_commitment() {
        let salt = 12345u64;

        // Board 1: Complete board with Cruiser horizontal
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1.place_ship(Ship::new(ShipKind::Cruiser, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");
        board1.place_ship(Ship::new(ShipKind::Destroyer, 2, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");

        // Board 2: Complete board with Cruiser vertical
        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2.place_ship(Ship::new(ShipKind::Cruiser, 0, 0, Orientation::Vertical))
            .expect("Ship placement should succeed");
        board2.place_ship(Ship::new(ShipKind::Destroyer, 0, 3, Orientation::Horizontal))
            .expect("Ship placement should succeed");

        let board1_array = board1.to_array().expect("Board should be ready");
        let board2_array = board2.to_array().expect("Board should be ready");

        let tree1 = BoardMerkleTree::build(board1_array, salt);
        let tree2 = BoardMerkleTree::build(board2_array, salt);

        assert_ne!(tree1.root(), tree2.root(), "Different ship orientations should produce different commitments");
    }

    #[test]
    fn test_commitment_with_multiple_ships() {
        let salt = 12345u64;

        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board.place_ship(Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal))
            .expect("Destroyer placement should succeed");
        board.place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Cruiser placement should succeed");

        let board_array = board.to_array().expect("Board should be ready");

        let tree = BoardMerkleTree::build(board_array.clone(), salt);
        let root = tree.root();

        assert_ne!(root, Felt::ZERO, "Root with multiple ships should not be zero");

        // Verify determinism
        let tree2 = BoardMerkleTree::build(board_array, salt);
        assert_eq!(root, tree2.root(), "Multiple builds should produce same root");
    }

    #[test]
    fn test_board_size_affects_commitment() {
        let salt = 12345u64;

        // Complete 6x6 board
        let mut board6x6 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board6x6.place_ship(Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");
        board6x6.place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        // Complete 8x8 board
        let mut board8x8 = Board::new(BoardSize::Smaller(SmallerBoardSize::EightByEight));
        board8x8.place_ship(Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");
        board8x8.place_ship(Ship::new(ShipKind::Cruiser, 2, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let board6x6_array = board6x6.to_array().expect("Board should be ready");
        let board8x8_array = board8x8.to_array().expect("Board should be ready");

        let tree1 = BoardMerkleTree::build(board6x6_array, salt);
        let tree2 = BoardMerkleTree::build(board8x8_array, salt);

        assert_ne!(tree1.root(), tree2.root(), "Different board sizes should produce different commitments");
    }

    #[test]
    fn test_commitment_is_not_trivial() {
        // Ensure commitment is not just hash of first element or something trivial
        let salt = 99999u64;

        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board.place_ship(Ship::new(ShipKind::Destroyer, 4, 4, Orientation::Horizontal))
            .expect("Ship placement should succeed");
        board.place_ship(Ship::new(ShipKind::Cruiser, 0, 3, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let board_array = board.to_array().expect("Board should be ready");

        let tree = BoardMerkleTree::build(board_array, salt);
        let root = tree.root();

        // Commitment should not equal hash of any single cell
        let first_cell = Felt::from(0u8);
        let first_cell_hash = pedersen_hash(&first_cell, &Felt::from(salt));

        assert_ne!(root, first_cell_hash, "Commitment should not be trivial single-cell hash");
    }
}
