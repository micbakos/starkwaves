pub mod events;
pub mod game;
pub mod starkwaves;

#[cfg(test)]
mod tests;
pub mod types;
use core::dict::Felt252Dict;
use merkle::{compute_merkle_root, verify};
use types::{Orientation, Ship, ShipKindTrait, create_board};

pub fn validate_and_commit(ships: Array<Ship>, board_size: u8, salt: felt252) -> felt252 {
    let ships_span = ships.span();
    assert_board_size(board_size);

    assert_eligible_ships(ships_span, board_size);
    assert_ships_fit_in_board(ships_span, board_size);

    let board = create_board(ships_span, board_size);

    return compute_merkle_root(board, salt);
}

pub fn verify_report(
    salted_status: felt252, proof: Array<felt252>, root: felt252, index: usize,
) -> bool {
    verify(salted_status, proof, root, index)
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
