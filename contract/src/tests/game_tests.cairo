use starknet::ContractAddress;
use crate::game::GameTrait;
use crate::types::{BoardSize, BoardSizeTrait, LargerBoardSize, SmallerBoardSize};

// Test helper functions
fn player_a() -> ContractAddress {
    0x1.try_into().unwrap()
}

fn player_b() -> ContractAddress {
    0x2.try_into().unwrap()
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
    assert!(game.player_a_bombs_on_b.len() == 0, "Player A bombs should be empty");
    assert!(game.player_b_bombs_on_a.len() == 0, "Player B bombs should be empty");
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
