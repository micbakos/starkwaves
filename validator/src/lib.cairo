#[cfg(test)]
mod tests;
pub mod types;
use core::array::ArrayTrait;
use core::dict::Felt252Dict;
use core::poseidon::poseidon_hash_span;
use types::{Orientation, Ship, ShipKindTrait};

pub fn validate_and_commit(ships: Array<Ship>, board_size: u8, salt: felt252) -> felt252 {
    let ships_span = ships.span();
    assert_board_size(board_size);

    assert_eligible_ships(ships_span, board_size);
    assert_ships_fit_in_board(ships_span, board_size);

    let board = assert_no_collisions_in_board(ships_span, board_size);

    return create_commitment(board, salt);
}

fn assert_board_size(board_size: u8) {
    assert!(
        board_size == 6 // 6x6
            || board_size == 8 // 8x8
            || board_size == 10 // 10x10
            || board_size == 12 // 12x12
            || board_size == 14 // 14x14
            || board_size == 20, // 20x20
        "Board is not a valid size.",
    );
}

fn assert_eligible_ships(ships: Span<Ship>, board_size: u8) {
    let mut ship_occurences: Felt252Dict<u8> = Default::default();

    for ship in ships {
        let id: felt252 = ship.kind.id().into();
        let occurences = ship_occurences.get(id) + 1;
        ship_occurences.insert(id, occurences);
    }

    let all_ship_kinds = ShipKindTrait::all();
    for ship_kind in all_ship_kinds {
        let id: felt252 = ship_kind.id().into();
        let occupied = ship_occurences.get(id);
        assert!(
            ship_kind.is_eligible(board_size, occupied),
            "There are {} occurences of ship id {} in a board of size ({}x{})",
            occupied,
            ship_kind,
            board_size,
            board_size,
        )
    }
}

fn assert_ships_fit_in_board(ships: Span<Ship>, board_size: u8) {
    for ship in ships {
        match ship.orientation {
            Orientation::Horizontal => {
                assert!(
                    *ship.x < board_size,
                    "Ship {} is out of bounds. Originating in [{}, {}]",
                    ship.kind,
                    ship.x,
                    ship.y,
                )

                assert!(
                    *ship.y + ship.kind.length() - 1 < board_size,
                    "Ship {} is out of bounds. Originating in [{}, {}]",
                    ship.kind,
                    ship.x,
                    ship.y,
                );
            },
            Orientation::Vertical => {
                assert!(
                    *ship.x + ship.kind.length() - 1 < board_size,
                    "Ship {} is out of bounds. Originating in [{}, {}]",
                    ship.kind,
                    ship.x,
                    ship.y,
                );
                assert!(
                    *ship.y < board_size,
                    "Ship {} is out of bounds. Originating in [{}, {}]",
                    ship.kind,
                    ship.x,
                    ship.y,
                )
            },
        }
    }
}

fn assert_no_collisions_in_board(ships: Span<Ship>, board_size: u8) -> Span<u8> {
    let mut board: Felt252Dict<u8> = Default::default();

    let offset = |x: u8, y: u8| -> felt252 {
        let rows_offset: u32 = x.into() * board_size.into();
        (rows_offset + y.into()).into()
    };

    for ship in ships {
        let id = ship.kind.id();
        let size = ship.kind.length();

        for step in 0..size {
            let (x, y) = match ship.orientation {
                Orientation::Horizontal => (*ship.x, *ship.y + step),
                Orientation::Vertical => (*ship.x + step, *ship.y),
            };

            let offset = offset(x, y);
            let item = board.get(offset);

            assert!(item == 0, "Ship {} collides with {} in [{},{}]", id, item, x, y)

            board.insert(offset, id);
        }
    }

    let mut board_array: Array<u8> = ArrayTrait::new();
    let array_size: u32 = board_size.into() * board_size.into();
    for i in 0..array_size {
        board_array.append(board.get(i.into()));
    }

    return board_array.span();
}

/// Computes a merkle root from a span of leaves using Poseidon hash
/// Each pair of leaves is hashed together, then the process repeats until one root remains
fn compute_merkle_root(mut leaves: Span<felt252>) -> felt252 {
    let len = leaves.len();

    if len == 0 {
        return 0;
    }

    if len == 1 {
        return *leaves.at(0);
    }

    let mut parent_tree_level: Array<felt252> = array![];
    let mut i = 0;

    while i < len {
        if i + 1 < len {
            let left = *leaves.at(i);
            let right = *leaves.at(i + 1);
            let pair = array![left, right];
            let hash = poseidon_hash_span(pair.span());
            parent_tree_level.append(hash);
            i += 2;
        } else {
            parent_tree_level.append(*leaves.at(i));
            i += 1;
        }
    }

    compute_merkle_root(parent_tree_level.span())
}

/// Creates a commitment by computing merkle root of board cells with salt
/// The salt is included as the last leaf to make the commitment binding and hiding
fn create_commitment(board: Span<u8>, salt: felt252) -> felt252 {
    let mut leaves: Array<felt252> = array![];

    // Convert board cells to felt252 and add as leaves
    let mut i = 0;
    while i < board.len() {
        leaves.append((*board.at(i)).into());
        i += 1;
    }

    // Add salt as final leaf
    leaves.append(salt);

    // Compute and return merkle root
    compute_merkle_root(leaves.span())
}
