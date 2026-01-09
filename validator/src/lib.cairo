#[cfg(test)]
mod tests;
pub mod types;
use core::array::ArrayTrait;
use core::dict::Felt252Dict;
use types::{Orientation, Ship, ShipKindTrait};

pub fn validate_and_commit(ships: Array<Ship>, board_size: u8, salt: felt252) -> felt252 {
    let ships_span = ships.span();
    assert_board_size(board_size);

    assert_eligible_ships(ships_span, board_size);
    assert_ships_fit_in_board(ships_span, board_size);

    let board = assert_no_collisions_in_board(ships_span, board_size);

    return compute_merkle_root(board, salt);
}

pub fn compute_merkle_root(board: Array<u8>, salt: felt252) -> felt252 {
    let mut leaves: Array<felt252> = array![];
    let board_span = board.span();
    let mut i = 0;
    while i < board_span.len() {
        let cell: felt252 = (*board_span.at(i)).into();
        let hash = core::pedersen::pedersen(cell, salt);
        leaves.append(hash);
        i += 1;
    }

    build_merkle_tree(leaves.span())
}

/// Recursively builds a Merkle tree from leaves using Pedersen hash
/// Pairs up leaves and hashes them together until one root remains
fn build_merkle_tree(leaves: Span<felt252>) -> felt252 {
    let len = leaves.len();

    // Base cases
    if len == 0 {
        return 0;
    }

    if len == 1 {
        return *leaves.at(0);
    }

    // Build next level by pairing and hashing
    let mut parent_level: Array<felt252> = array![];
    let mut i = 0;

    while i < len {
        if i + 1 < len {
            // Hash pair using Pedersen
            let left = *leaves.at(i);
            let right = *leaves.at(i + 1);
            let hash = core::pedersen::pedersen(left, right);
            parent_level.append(hash);
            i += 2;
        } else {
            // Odd node: carry forward unchanged (matches rs_merkle behavior)
            parent_level.append(*leaves.at(i));
            i += 1;
        }
    }

    // Recurse on parent level
    build_merkle_tree(parent_level.span())
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

fn assert_no_collisions_in_board(ships: Span<Ship>, board_size: u8) -> Array<u8> {
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

    return board_array;
}
