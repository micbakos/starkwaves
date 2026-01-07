use starkwaves_validator::types::{Orientation, Ship, ShipKind};
use starkwaves_validator::validate_and_commit;

// ===============================
// Valid Board Tests
// ===============================

#[test]
fn test_validate_and_commit_6x6_valid() {
    let ships = array![
        Ship { kind: ShipKind::Destroyer, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Cruiser, x: 2, y: 1, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 12345;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Valid 6x6 board should produce non-zero commitment");
}

#[test]
fn test_validate_and_commit_8x8_valid() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 0, y: 3, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 54321;
    let commitment = validate_and_commit(ships, 8, salt);
    assert!(commitment != 0, "Valid 8x8 board should produce non-zero commitment");
}

#[test]
fn test_validate_and_commit_10x10_valid() {
    let ships = array![
        Ship { kind: ShipKind::Carrier, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Battleship, x: 0, y: 9, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Cruiser, x: 3, y: 2, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Submarine, x: 7, y: 6, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Destroyer, x: 8, y: 9, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 99999;
    let commitment = validate_and_commit(ships, 10, salt);
    assert!(commitment != 0, "Valid 10x10 board should produce non-zero commitment");
}

#[test]
fn test_validate_and_commit_12x12_valid() {
    let ships = array![
        Ship { kind: ShipKind::SuperCarrier, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Carrier, x: 2, y: 1, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Battleship, x: 3, y: 4, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Cruiser, x: 2, y: 5, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Submarine, x: 3, y: 10, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Submarine, x: 7, y: 6, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 8, y: 2, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 9, y: 7, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 77777;
    let commitment = validate_and_commit(ships, 12, salt);
    assert!(commitment != 0, "Valid 12x12 board should produce non-zero commitment");
}

#[test]
fn test_validate_and_commit_vertical_ships() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Destroyer, x: 0, y: 2, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 11111;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Valid board with vertical ships should produce non-zero commitment");
}

#[test]
fn test_validate_and_commit_mixed_orientations() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 0, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 22222;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(
        commitment != 0, "Valid board with mixed orientations should produce non-zero commitment",
    );
}

// ===============================
// Invalid Board Size Tests
// ===============================

#[test]
#[should_panic(expected: "Board is not a valid size.")]
fn test_validate_and_commit_invalid_board_size_5() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 5, salt);
}

#[test]
#[should_panic(expected: "Board is not a valid size.")]
fn test_validate_and_commit_invalid_board_size_7() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 7, salt);
}

#[test]
#[should_panic(expected: "Board is not a valid size.")]
fn test_validate_and_commit_invalid_board_size_15() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 15, salt);
}

// ===============================
// Invalid Ship Configuration Tests
// ===============================

#[test]
#[should_panic]
fn test_validate_and_commit_wrong_ship_for_6x6() {
    let ships = array![
        Ship { kind: ShipKind::Carrier, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 0, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 6, salt);
}

#[test]
#[should_panic]
fn test_validate_and_commit_too_many_cruisers() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Cruiser, x: 0, y: 1, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 6, salt);
}

#[test]
#[should_panic]
fn test_validate_and_commit_super_carrier_on_10x10() {
    let ships = array![
        Ship { kind: ShipKind::SuperCarrier, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 0, y: 2, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 10, salt);
}

#[test]
#[should_panic]
fn test_validate_and_commit_missing_required_ships() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 6, salt);
}

// ===============================
// Out of Bounds Tests
// ===============================

#[test]
#[should_panic]
fn test_validate_and_commit_ship_out_of_bounds_horizontal() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 5, y: 5, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 0, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 6, salt);
}

#[test]
#[should_panic]
fn test_validate_and_commit_ship_out_of_bounds_vertical() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 5, y: 0, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 6, salt);
}

#[test]
#[should_panic]
fn test_validate_and_commit_ship_y_out_of_bounds() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 10, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 0, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 6, salt);
}

// ===============================
// Collision Tests
// ===============================

#[test]
#[should_panic]
fn test_validate_and_commit_horizontal_collision() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 0, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 10, salt);
}

#[test]
#[should_panic]
fn test_validate_and_commit_vertical_collision() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 0, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 10, salt);
}

#[test]
#[should_panic]
fn test_validate_and_commit_perpendicular_collision() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 1, y: 0, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 10, salt);
}

// ===============================
// Salt Tests
// ===============================

#[test]
fn test_validate_and_commit_different_salts_produce_different_commitments() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 0, orientation: Orientation::Vertical },
    ];

    let salt1: felt252 = 11111;
    let salt2: felt252 = 99999;

    let commitment1 = validate_and_commit(ships.clone(), 6, salt1);
    let commitment2 = validate_and_commit(ships, 6, salt2);

    assert!(commitment1 != commitment2, "Different salts should produce different commitments");
}

#[test]
fn test_validate_and_commit_same_input_produces_same_commitment() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 0, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 12345;

    let commitment1 = validate_and_commit(ships.clone(), 6, salt);
    let commitment2 = validate_and_commit(ships, 6, salt);

    assert!(commitment1 == commitment2, "Same inputs should produce same commitment");
}

#[test]
fn test_validate_and_commit_zero_salt() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 0, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 0;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Zero salt should still produce valid commitment");
}

// ===============================
// Edge Case Tests - Board Edges and Corners
// ===============================

#[test]
fn test_validate_and_commit_ships_at_corners() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 4, y: 4, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 33333;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Ships at corners should be valid");
}

#[test]
fn test_validate_and_commit_ships_at_right_edge() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 3, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 3, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 44444;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Ships at right edge should be valid");
}

#[test]
fn test_validate_and_commit_ships_at_bottom_edge() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 3, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Destroyer, x: 3, y: 4, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 55555;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Ships at bottom edge should be valid");
}

#[test]
fn test_validate_and_commit_ship_spanning_full_width() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 2, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 66666;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Ship spanning width should be valid");
}

// ===============================
// Edge Case Tests - Adjacent Ships (No Collision)
// ===============================

#[test]
fn test_validate_and_commit_ships_touching_horizontally() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 0, y: 3, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 77777;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Ships touching but not colliding should be valid");
}

#[test]
fn test_validate_and_commit_ships_touching_vertically() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Destroyer, x: 3, y: 0, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 88888;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Ships touching vertically should be valid");
}

#[test]
fn test_validate_and_commit_ships_diagonal_adjacent() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 1, y: 1, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 99000;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Diagonally adjacent ships should be valid");
}

// ===============================
// Edge Case Tests - Larger Boards
// ===============================

#[test]
fn test_validate_and_commit_14x14_valid() {
    let ships = array![
        Ship { kind: ShipKind::SuperCarrier, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Carrier, x: 1, y: 1, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Battleship, x: 2, y: 7, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Cruiser, x: 5, y: 3, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Submarine, x: 8, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Submarine, x: 10, y: 5, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Destroyer, x: 9, y: 10, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 12, y: 8, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 141414;
    let commitment = validate_and_commit(ships, 14, salt);
    assert!(commitment != 0, "Valid 14x14 board should produce non-zero commitment");
}

#[test]
fn test_validate_and_commit_20x20_valid() {
    let ships = array![
        Ship { kind: ShipKind::SuperCarrier, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Carrier, x: 5, y: 5, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Battleship, x: 10, y: 10, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Cruiser, x: 15, y: 0, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Submarine, x: 0, y: 15, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Submarine, x: 17, y: 10, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Destroyer, x: 8, y: 8, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 12, y: 18, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 202020;
    let commitment = validate_and_commit(ships, 20, salt);
    assert!(commitment != 0, "Valid 20x20 board should produce non-zero commitment");
}

// ===============================
// Edge Case Tests - Salt Values
// ===============================

#[test]
fn test_validate_and_commit_large_salt() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 0, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 999999999999999999;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Large salt should produce valid commitment");
}

#[test]
fn test_validate_and_commit_negative_salt() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 0, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = -1;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Negative salt should produce valid commitment");
}

// ===============================
// Edge Case Tests - Ship Positioning
// ===============================

#[test]
fn test_validate_and_commit_all_ships_same_column() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 1, y: 0, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 11223;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "All ships in same column should be valid");
}

#[test]
fn test_validate_and_commit_all_ships_same_row() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Destroyer, x: 0, y: 4, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 33445;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "All ships in same row should be valid");
}

#[test]
fn test_validate_and_commit_complex_layout_12x12() {
    let ships = array![
        Ship { kind: ShipKind::SuperCarrier, x: 5, y: 5, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Carrier, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Battleship, x: 6, y: 8, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Cruiser, x: 3, y: 1, orientation: Orientation::Vertical },
        Ship { kind: ShipKind::Submarine, x: 2, y: 2, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Submarine, x: 9, y: 9, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 0, y: 10, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 10, y: 2, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 121212;
    let commitment = validate_and_commit(ships, 12, salt);
    assert!(commitment != 0, "Complex layout should produce valid commitment");
}

// ===============================
// Edge Case Tests - Boundary Conditions
// ===============================

#[test]
#[should_panic]
fn test_validate_and_commit_ship_one_cell_over_boundary_horizontal() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 4, y: 4, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 0, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 6, salt);
}

#[test]
#[should_panic]
fn test_validate_and_commit_ship_one_cell_over_boundary_vertical() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 5, y: 2, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 6, salt);
}

#[test]
fn test_validate_and_commit_exact_fit_horizontal() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 3, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 2, y: 4, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 98765;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Ship fitting exactly at boundary should be valid");
}

#[test]
fn test_validate_and_commit_exact_fit_vertical() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 0, y: 4, orientation: Orientation::Vertical },
    ];
    let salt: felt252 = 56789;
    let commitment = validate_and_commit(ships, 6, salt);
    assert!(commitment != 0, "Ship fitting exactly at boundary should be valid");
}

// ===============================
// Edge Case Tests - Multiple Collisions
// ===============================

#[test]
#[should_panic]
fn test_validate_and_commit_triple_collision() {
    let ships = array![
        Ship { kind: ShipKind::Carrier, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Battleship, x: 0, y: 2, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Cruiser, x: 0, y: 4, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Submarine, x: 1, y: 1, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 0, y: 5, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 10, salt);
}

#[test]
#[should_panic]
fn test_validate_and_commit_exact_same_position() {
    let ships = array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 0, y: 0, orientation: Orientation::Horizontal },
    ];
    let salt: felt252 = 12345;
    validate_and_commit(ships, 10, salt);
}
