use crate::types::{BoardSize, LargerBoardSize, SmallerBoardSize};
use crate::utils::{
    append_bomb_at, bytes_to_offset, cartesian_as_bytes, cartesian_to_offset, contains_bomb_at,
    get_bomb_offset_at_turn, offset_to_cartesian,
};

// ===============================
// cartesian_as_bytes() Tests
// ===============================

#[test]
fn test_cartesian_as_bytes_origin() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let (high, low) = cartesian_as_bytes(@size, 0, 0);
    assert!(high == 0 && low == 0, "Offset (0,0) should be (0,0)");
}

#[test]
fn test_cartesian_as_bytes_first_row() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let (high, low) = cartesian_as_bytes(@size, 0, 5);
    // offset = 0 * 6 + 5 = 5
    assert!(high == 0 && low == 5, "Offset (0,5) should be (0,5)");
}

#[test]
fn test_cartesian_as_bytes_second_row() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let (high, low) = cartesian_as_bytes(@size, 1, 0);
    // offset = 1 * 6 + 0 = 6
    assert!(high == 0 && low == 6, "Offset (1,0) should be (0,6)");
}

#[test]
fn test_cartesian_as_bytes_middle() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let (high, low) = cartesian_as_bytes(@size, 2, 3);
    // offset = 2 * 6 + 3 = 15
    assert!(high == 0 && low == 15, "Offset (2,3) should be (0,15)");
}

#[test]
fn test_cartesian_as_bytes_last_cell() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let (high, low) = cartesian_as_bytes(@size, 5, 5);
    // offset = 5 * 6 + 5 = 35
    assert!(high == 0 && low == 35, "Offset (5,5) should be (0,35)");
}

#[test]
fn test_cartesian_as_bytes_large_board() {
    let size = BoardSize::Larger(LargerBoardSize::TwentyByTwenty);
    let (high, low) = cartesian_as_bytes(@size, 19, 19);
    // offset = 19 * 20 + 19 = 399
    // 399 = 1 * 256 + 143
    assert!(high == 1 && low == 143, "Offset (19,19) on 20x20 should be (1,143)");
}

#[test]
fn test_cartesian_as_bytes_boundary_256() {
    let size = BoardSize::Larger(LargerBoardSize::TwentyByTwenty);
    let (high, low) = cartesian_as_bytes(@size, 12, 16);
    // offset = 12 * 20 + 16 = 256
    // 256 = 1 * 256 + 0
    assert!(high == 1 && low == 0, "Offset 256 should be (1,0)");
}

// ===============================
// offset_to_cartesian() Tests
// ===============================

#[test]
fn test_offset_to_cartesian_origin() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let (x, y) = offset_to_cartesian(@size, 0);
    assert!(x == 0 && y == 0, "Offset 0 should map to (0,0)");
}

#[test]
fn test_offset_to_cartesian_end_of_first_row() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let (x, y) = offset_to_cartesian(@size, 5);
    assert!(x == 0 && y == 5, "Offset 5 should map to (0,5)");
}

#[test]
fn test_offset_to_cartesian_start_of_second_row() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let (x, y) = offset_to_cartesian(@size, 6);
    assert!(x == 1 && y == 0, "Offset 6 should map to (1,0)");
}

// ===============================
// cartesian_to_offset() Tests
// ===============================

#[test]
fn test_cartesian_to_offset_origin() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let offset = cartesian_to_offset(@size, 0, 0);
    assert!(offset == 0, "Origin should be offset 0");
}

#[test]
fn test_cartesian_to_offset_last_cell() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let offset = cartesian_to_offset(@size, 5, 5);
    assert!(offset == 35, "(5,5) on 6x6 should be offset 35");
}

// ===============================
// bytes_to_offset() Tests
// ===============================

#[test]
fn test_bytes_to_offset_zero() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let offset = bytes_to_offset(@size, 0, 0);
    assert!(offset == 0, "(0,0) bytes should be offset 0");
}

#[test]
fn test_bytes_to_offset_high_byte() {
    let size = BoardSize::Larger(LargerBoardSize::TwentyByTwenty);
    let offset = bytes_to_offset(@size, 1, 143);
    // 1 * 256 + 143 = 399
    assert!(offset == 399, "(1,143) bytes should be offset 399");
}

// ===============================
// Roundtrip Tests
// ===============================

#[test]
fn test_offset_roundtrip_6x6() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let coords = array![(0_u8, 0_u8), (0, 5), (1, 0), (2, 3), (5, 5)];

    let mut i = 0;
    while i < coords.len() {
        let (x, y) = *coords.at(i);
        let offset = cartesian_to_offset(@size, x, y);
        let (rx, ry) = offset_to_cartesian(@size, offset);
        assert!(rx == x && ry == y, "Roundtrip should preserve coords");
        i += 1;
    };
}

#[test]
fn test_offset_roundtrip_20x20() {
    let size = BoardSize::Larger(LargerBoardSize::TwentyByTwenty);
    let coords = array![(0_u8, 0_u8), (10, 10), (19, 19), (12, 16)];

    let mut i = 0;
    while i < coords.len() {
        let (x, y) = *coords.at(i);
        let offset = cartesian_to_offset(@size, x, y);
        let (rx, ry) = offset_to_cartesian(@size, offset);
        assert!(rx == x && ry == y, "Large board roundtrip should work");
        i += 1;
    };
}

#[test]
fn test_bytes_roundtrip() {
    let size = BoardSize::Larger(LargerBoardSize::TwentyByTwenty);
    let coords = array![(0_u8, 0_u8), (10, 10), (19, 19), (12, 16)];

    let mut i = 0;
    while i < coords.len() {
        let (x, y) = *coords.at(i);
        let (high, low) = cartesian_as_bytes(@size, x, y);
        let offset = bytes_to_offset(@size, high, low);
        let (rx, ry) = offset_to_cartesian(@size, offset);
        assert!(rx == x && ry == y, "Bytes roundtrip should preserve coords");
        i += 1;
    };
}

// ===============================
// contains_bomb_at() Tests
// ===============================

#[test]
fn test_contains_bomb_at_empty() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let bombs: ByteArray = Default::default();
    assert!(!contains_bomb_at(@bombs, @size, 0, 0), "Empty ByteArray should not contain bomb");
}

#[test]
fn test_contains_bomb_at_single_match() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let mut bombs: ByteArray = Default::default();
    append_bomb_at(ref bombs, @size, 0, 5);

    assert!(contains_bomb_at(@bombs, @size, 0, 5), "Should find bomb at (0,5)");
}

#[test]
fn test_contains_bomb_at_single_no_match() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let mut bombs: ByteArray = Default::default();
    append_bomb_at(ref bombs, @size, 0, 5);

    assert!(!contains_bomb_at(@bombs, @size, 0, 6), "Should not find bomb at (0,6)");
    assert!(!contains_bomb_at(@bombs, @size, 1, 5), "Should not find bomb at (1,5)");
}

#[test]
fn test_contains_bomb_at_multiple() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let mut bombs: ByteArray = Default::default();
    append_bomb_at(ref bombs, @size, 0, 5);
    append_bomb_at(ref bombs, @size, 2, 3);
    append_bomb_at(ref bombs, @size, 5, 5);

    assert!(contains_bomb_at(@bombs, @size, 0, 5), "Should find first bomb");
    assert!(contains_bomb_at(@bombs, @size, 2, 3), "Should find second bomb");
    assert!(contains_bomb_at(@bombs, @size, 5, 5), "Should find third bomb");
    assert!(!contains_bomb_at(@bombs, @size, 1, 1), "Should not find non-existent bomb");
}

#[test]
fn test_contains_bomb_at_odd_length() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let mut bombs: ByteArray = Default::default();
    bombs.append_byte(0);
    // Incomplete pair - should not crash

    assert!(!contains_bomb_at(@bombs, @size, 0, 0), "Should handle incomplete pair gracefully");
}

// ===============================
// get_bomb_offset_at_turn() Tests
// ===============================

#[test]
fn test_get_bomb_offset_at_turn_empty() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let bombs: ByteArray = Default::default();
    assert!(get_bomb_offset_at_turn(@bombs, @size, 0).is_none(), "Should return None when empty");
}

#[test]
fn test_get_bomb_offset_at_turn_first() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let mut bombs: ByteArray = Default::default();
    append_bomb_at(ref bombs, @size, 2, 3);

    let offset = get_bomb_offset_at_turn(@bombs, @size, 0);
    assert!(offset.is_some(), "Should return Some for turn 0");
    // offset = 2 * 6 + 3 = 15
    assert!(offset.unwrap() == 15, "Turn 0 offset should be 15");
}

#[test]
fn test_get_bomb_offset_at_turn_multiple() {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let mut bombs: ByteArray = Default::default();
    append_bomb_at(ref bombs, @size, 0, 0);
    append_bomb_at(ref bombs, @size, 3, 4);

    let first = get_bomb_offset_at_turn(@bombs, @size, 0);
    let second = get_bomb_offset_at_turn(@bombs, @size, 1);
    let third = get_bomb_offset_at_turn(@bombs, @size, 2);

    assert!(first.unwrap() == 0, "Turn 0 should be offset 0");
    // 3 * 6 + 4 = 22
    assert!(second.unwrap() == 22, "Turn 1 should be offset 22");
    assert!(third.is_none(), "Turn 2 should be None");
}
