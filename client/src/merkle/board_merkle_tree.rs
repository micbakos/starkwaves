use crate::merkle::pedersen_hasher::PedersenHasher;
use crate::types::Board;
use rs_merkle::MerkleTree;
use starknet::core::crypto::pedersen_hash;
use starknet::core::types::Felt;

pub struct BoardMerkleTree {
    tree: MerkleTree<PedersenHasher>,
}

/// Produces a Merkle Tree that uses Pedersen hash for node hashing
impl BoardMerkleTree {
    pub fn build(board: &Board, salt: u64) -> Self {
        let salt = Felt::from(salt);

        let leaves = board
            .to_array()
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
    use crate::types::{Ship, ShipKind, Orientation};
    use crate::types::board_size::{BoardSize, SmallerBoardSize};

    #[test]
    fn test_empty_board_commitment() {
        let board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        let salt = 12345u64;

        let tree = BoardMerkleTree::build(&board, salt);
        let root = tree.root();

        assert_ne!(root, Felt::ZERO, "Root should not be zero");
    }

    #[test]
    fn test_same_board_same_salt_produces_same_commitment() {
        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board.place_ship(Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");

        let salt = 12345u64;

        let tree1 = BoardMerkleTree::build(&board, salt);
        let tree2 = BoardMerkleTree::build(&board, salt);

        assert_eq!(tree1.root(), tree2.root(), "Same board and salt should produce same commitment");
    }

    #[test]
    fn test_different_salt_produces_different_commitment() {
        let board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));

        let tree1 = BoardMerkleTree::build(&board, 11111);
        let tree2 = BoardMerkleTree::build(&board, 22222);

        assert_ne!(tree1.root(), tree2.root(), "Different salts should produce different commitments");
    }

    #[test]
    fn test_different_boards_produce_different_commitments() {
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1.place_ship(Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");

        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2.place_ship(Ship::new(ShipKind::Cruiser, 1, 1, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let salt = 12345u64;

        let tree1 = BoardMerkleTree::build(&board1, salt);
        let tree2 = BoardMerkleTree::build(&board2, salt);

        assert_ne!(tree1.root(), tree2.root(), "Different boards should produce different commitments");
    }

    #[test]
    fn test_ship_position_affects_commitment() {
        let salt = 12345u64;

        // Same ship, different position
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1.place_ship(Ship::new(ShipKind::Destroyer, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");

        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2.place_ship(Ship::new(ShipKind::Destroyer, 2, 2, Orientation::Horizontal))
            .expect("Ship placement should succeed");

        let tree1 = BoardMerkleTree::build(&board1, salt);
        let tree2 = BoardMerkleTree::build(&board2, salt);

        assert_ne!(tree1.root(), tree2.root(), "Different ship positions should produce different commitments");
    }

    #[test]
    fn test_ship_orientation_affects_commitment() {
        let salt = 12345u64;

        // Same ship and position, different orientation
        let mut board1 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board1.place_ship(Ship::new(ShipKind::Cruiser, 0, 0, Orientation::Horizontal))
            .expect("Ship placement should succeed");

        let mut board2 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board2.place_ship(Ship::new(ShipKind::Cruiser, 0, 0, Orientation::Vertical))
            .expect("Ship placement should succeed");

        let tree1 = BoardMerkleTree::build(&board1, salt);
        let tree2 = BoardMerkleTree::build(&board2, salt);

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

        let tree = BoardMerkleTree::build(&board, salt);
        let root = tree.root();

        assert_ne!(root, Felt::ZERO, "Root with multiple ships should not be zero");

        // Verify determinism
        let tree2 = BoardMerkleTree::build(&board, salt);
        assert_eq!(root, tree2.root(), "Multiple builds should produce same root");
    }

    #[test]
    fn test_board_size_affects_commitment() {
        let salt = 12345u64;

        let board6x6 = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        let board8x8 = Board::new(BoardSize::Smaller(SmallerBoardSize::EightByEight));

        let tree1 = BoardMerkleTree::build(&board6x6, salt);
        let tree2 = BoardMerkleTree::build(&board8x8, salt);

        assert_ne!(tree1.root(), tree2.root(), "Different board sizes should produce different commitments");
    }

    #[test]
    fn test_commitment_is_not_trivial() {
        // Ensure commitment is not just hash of first element or something trivial
        let salt = 99999u64;

        let mut board = Board::new(BoardSize::Smaller(SmallerBoardSize::SixBySix));
        board.place_ship(Ship::new(ShipKind::Destroyer, 4, 4, Orientation::Horizontal))
            .expect("Ship placement should succeed");

        let tree = BoardMerkleTree::build(&board, salt);
        let root = tree.root();

        // Commitment should not equal hash of any single cell
        let first_cell = Felt::from(0u8);
        let first_cell_hash = pedersen_hash(&first_cell, &Felt::from(salt));

        assert_ne!(root, first_cell_hash, "Commitment should not be trivial single-cell hash");
    }
}
