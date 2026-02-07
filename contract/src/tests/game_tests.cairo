use starknet::ContractAddress;
use crate::game::{GameBombsTrait, GameTrait};
use crate::types::{BoardSize, BoardSizeTrait, LargerBoardSize, SmallerBoardSize};

// Test helper functions
fn player_a() -> ContractAddress {
    0x1.try_into().unwrap()
}

fn player_b() -> ContractAddress {
    0x2.try_into().unwrap()
}

fn player_c() -> ContractAddress {
    0x3.try_into().unwrap()
}

// ===============================
// Game::new() Tests
// ===============================

#[test]
fn test_new_game_6x6() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);
    assert!(game.id == 1, "Game ID should be 1");
    assert!(game.board_size.size() == 6, "Board size should be 6");
    assert!(game.player_a == player_a(), "Player A should match");
    assert!(game.player_b == player_b(), "Player B should match");
    assert!(game.player_a_root.is_none(), "Player A root should be None");
    assert!(game.player_b_root.is_none(), "Player B root should be None");
    assert!(game.attacking_player.is_none(), "Attacking player should be None");
    assert!(game.turn_index == 0, "Turn index should be 0");
    assert!(game.player_a_bombs.len() == 0, "Player A bombs should be empty");
    assert!(game.player_b_bombs.len() == 0, "Player B bombs should be empty");
}

#[test]
fn test_new_game_8x8() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::EightByEight);
    let game = GameTrait::new(2, player_a(), player_b(), board_size);
    assert!(game.board_size.size() == 8, "Board size should be 8");
}

#[test]
fn test_new_game_10x10() {
    let game = GameTrait::new(3, player_a(), player_b(), BoardSize::Standard);
    assert!(game.board_size.size() == 10, "Board size should be 10");
}

#[test]
fn test_new_game_12x12() {
    let board_size = BoardSize::Larger(LargerBoardSize::TwelveByTwelve);
    let game = GameTrait::new(4, player_a(), player_b(), board_size);
    assert!(game.board_size.size() == 12, "Board size should be 12");
}

#[test]
fn test_new_game_14x14() {
    let board_size = BoardSize::Larger(LargerBoardSize::FourteenByFourteen);
    let game = GameTrait::new(5, player_a(), player_b(), board_size);
    assert!(game.board_size.size() == 14, "Board size should be 14");
}

#[test]
fn test_new_game_20x20() {
    let board_size = BoardSize::Larger(LargerBoardSize::TwentyByTwenty);
    let game = GameTrait::new(6, player_a(), player_b(), board_size);
    assert!(game.board_size.size() == 20, "Board size should be 20");
}

// ===============================
// Game::commit_root() Tests
// ===============================

#[test]
fn test_commit_root_player_a() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let mut game = GameTrait::new(1, player_a(), player_b(), board_size);
    let root: felt252 = 0x123456;

    game.commit_root(player_a(), root);

    assert!(game.player_a_root.is_some(), "Player A root should be set");
    assert!(game.player_a_root.unwrap() == root, "Player A root should match");
    assert!(game.player_b_root.is_none(), "Player B root should still be None");
    assert!(game.attacking_player.is_none(), "Attacking player should still be None");
}

#[test]
fn test_commit_root_player_b() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let mut game = GameTrait::new(1, player_a(), player_b(), board_size);
    let root: felt252 = 0xabcdef;

    game.commit_root(player_b(), root);

    assert!(game.player_b_root.is_some(), "Player B root should be set");
    assert!(game.player_b_root.unwrap() == root, "Player B root should match");
    assert!(game.player_a_root.is_none(), "Player A root should still be None");
    assert!(game.attacking_player.is_none(), "Attacking player should still be None");
}

#[test]
fn test_commit_root_both_players() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let mut game = GameTrait::new(1, player_a(), player_b(), board_size);
    let root_a: felt252 = 0x111111;
    let root_b: felt252 = 0x222222;

    game.commit_root(player_a(), root_a);
    game.commit_root(player_b(), root_b);

    assert!(game.player_a_root.is_some(), "Player A root should be set");
    assert!(game.player_b_root.is_some(), "Player B root should be set");
    assert!(game.attacking_player.is_some(), "Attacking player should be set");
    assert!(game.attacking_player.unwrap() == player_a(), "Player A should attack first");
}

#[test]
fn test_commit_root_both_players_reverse_order() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let mut game = GameTrait::new(1, player_a(), player_b(), board_size);
    let root_a: felt252 = 0x111111;
    let root_b: felt252 = 0x222222;

    game.commit_root(player_b(), root_b);
    game.commit_root(player_a(), root_a);

    assert!(game.attacking_player.is_some(), "Attacking player should be set");
    assert!(game.attacking_player.unwrap() == player_a(), "Player A should attack first");
}

// ===============================
// offset_bytes() Tests
// ===============================

#[test]
fn test_offset_bytes_origin() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);
    let (high, low) = game.offset_bytes(0, 0);
    assert!(high == 0 && low == 0, "Offset (0,0) should be (0,0)");
}

#[test]
fn test_offset_bytes_first_row() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);
    let (high, low) = game.offset_bytes(0, 5);
    // offset = 0 * 6 + 5 = 5
    assert!(high == 0 && low == 5, "Offset (0,5) should be (0,5)");
}

#[test]
fn test_offset_bytes_second_row() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);
    let (high, low) = game.offset_bytes(1, 0);
    // offset = 1 * 6 + 0 = 6
    assert!(high == 0 && low == 6, "Offset (1,0) should be (0,6)");
}

#[test]
fn test_offset_bytes_middle() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);
    let (high, low) = game.offset_bytes(2, 3);
    // offset = 2 * 6 + 3 = 15
    assert!(high == 0 && low == 15, "Offset (2,3) should be (0,15)");
}

#[test]
fn test_offset_bytes_last_cell() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);
    let (high, low) = game.offset_bytes(5, 5);
    // offset = 5 * 6 + 5 = 35
    assert!(high == 0 && low == 35, "Offset (5,5) should be (0,35)");
}

#[test]
fn test_offset_bytes_large_board() {
    let board_size = BoardSize::Larger(LargerBoardSize::TwentyByTwenty);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);
    let (high, low) = game.offset_bytes(19, 19);
    // offset = 19 * 20 + 19 = 399
    // 399 = 1 * 256 + 143
    assert!(high == 1 && low == 143, "Offset (19,19) on 20x20 should be (1,143)");
}

#[test]
fn test_offset_bytes_boundary_256() {
    let board_size = BoardSize::Larger(LargerBoardSize::TwentyByTwenty);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);
    let (high, low) = game.offset_bytes(12, 16);
    // offset = 12 * 20 + 16 = 256
    // 256 = 1 * 256 + 0
    assert!(high == 1 && low == 0, "Offset 256 should be (1,0)");
}

// ===============================
// bomb_in_current_turn() Tests
// ===============================

#[test]
fn test_bomb_in_current_turn_no_bombs() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);
    let bomb = game.bomb_offset_in_current_turn(@player_a());
    assert!(bomb.is_none(), "Should return None when no bombs");
}

#[test]
fn test_bomb_in_current_turn_wrong_player() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);
    let bomb = game.bomb_offset_in_current_turn(@player_c());
    assert!(bomb.is_none(), "Should return None for non-player");
}

// ===============================
// is_bombed() Tests
// ===============================

#[test]
fn test_is_bombed_empty() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);
    assert!(!game.is_bombed(@player_a(), 0, 0), "Should return false when no bombs");
    assert!(!game.is_bombed(@player_b(), 3, 3), "Should return false when no bombs");
}

#[test]
fn test_is_bombed_wrong_player() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);
    assert!(!game.is_bombed(@player_c(), 0, 0), "Should return false for non-player");
}

// ===============================
// contains_bomb() Tests
// ===============================

#[test]
fn test_contains_bomb_empty() {
    let bombs: ByteArray = Default::default();
    assert!(!bombs.contains_bomb(0, 0), "Empty ByteArray should not contain bomb");
}

#[test]
fn test_contains_bomb_single_bomb_match() {
    let mut bombs: ByteArray = Default::default();
    bombs.append_byte(0);
    bombs.append_byte(5);

    assert!(bombs.contains_bomb(0, 5), "Should find bomb at (0,5)");
}

#[test]
fn test_contains_bomb_single_bomb_no_match() {
    let mut bombs: ByteArray = Default::default();
    bombs.append_byte(0);
    bombs.append_byte(5);

    assert!(!bombs.contains_bomb(0, 6), "Should not find bomb at (0,6)");
    assert!(!bombs.contains_bomb(1, 5), "Should not find bomb at (1,5)");
}

#[test]
fn test_contains_bomb_multiple_bombs() {
    let mut bombs: ByteArray = Default::default();
    // Add bomb at offset 5
    bombs.append_byte(0);
    bombs.append_byte(5);
    // Add bomb at offset 15
    bombs.append_byte(0);
    bombs.append_byte(15);
    // Add bomb at offset 256
    bombs.append_byte(1);
    bombs.append_byte(0);

    assert!(bombs.contains_bomb(0, 5), "Should find first bomb");
    assert!(bombs.contains_bomb(0, 15), "Should find second bomb");
    assert!(bombs.contains_bomb(1, 0), "Should find third bomb");
    assert!(!bombs.contains_bomb(0, 10), "Should not find non-existent bomb");
}

#[test]
fn test_contains_bomb_odd_length() {
    let mut bombs: ByteArray = Default::default();
    bombs.append_byte(0);
    // Incomplete pair - should not crash

    assert!(!bombs.contains_bomb(0, 0), "Should handle incomplete pair gracefully");
}

// ===============================
// Integration Tests
// ===============================

#[test]
fn test_full_game_flow_basic() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let mut game = GameTrait::new(1, player_a(), player_b(), board_size);

    // Both players commit
    game.commit_root(player_a(), 0x111);
    game.commit_root(player_b(), 0x222);

    assert!(game.attacking_player.is_some(), "Game should be ready to start");
    assert!(game.attacking_player.unwrap() == player_a(), "Player A goes first");
}

#[test]
fn test_offset_roundtrip() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);

    // Test various coordinates
    let coords = array![(0_u8, 0_u8), (0, 5), (1, 0), (2, 3), (5, 5)];

    let mut i = 0;
    while i < coords.len() {
        let (x, y) = *coords.at(i);
        let (high, low) = game.offset_bytes(x, y);

        // Reconstruct offset
        let offset: u32 = high.into() * 256 + low.into();
        let size: u32 = game.board_size.size().into();
        let reconstructed_x: u8 = (offset / size).try_into().unwrap();
        let reconstructed_y: u8 = (offset % size).try_into().unwrap();

        assert!(reconstructed_x == x && reconstructed_y == y, "Roundtrip should preserve coords");

        i += 1;
    };
}

#[test]
fn test_offset_roundtrip_large_board() {
    let board_size = BoardSize::Larger(LargerBoardSize::TwentyByTwenty);
    let game = GameTrait::new(1, player_a(), player_b(), board_size);

    let coords = array![(0_u8, 0_u8), (10, 10), (19, 19), (12, 16)];

    let mut i = 0;
    while i < coords.len() {
        let (x, y) = *coords.at(i);
        let (high, low) = game.offset_bytes(x, y);

        let offset: u32 = high.into() * 256 + low.into();
        let size: u32 = game.board_size.size().into();
        let reconstructed_x: u8 = (offset / size).try_into().unwrap();
        let reconstructed_y: u8 = (offset % size).try_into().unwrap();

        assert!(reconstructed_x == x && reconstructed_y == y, "Large board roundtrip should work");

        i += 1;
    };
}
