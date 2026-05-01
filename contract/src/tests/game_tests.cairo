use starknet::ContractAddress;
use crate::game::{Game, GameTrait};
use crate::types::phase::{COMMIT_TIMEOUT_SECS, REVEAL_TIMEOUT_SECS, TURN_TIMEOUT_SECS};
use crate::types::{BoardSize, BoardSizeTrait, LargerBoardSize, Outcome, SmallerBoardSize};

// Test helper functions
fn player_a() -> ContractAddress {
    0x1.try_into().unwrap()
}

fn player_b() -> ContractAddress {
    0x2.try_into().unwrap()
}

// Mirror of constants in types/phase.cairo. Kept in sync manually because
// the source constants are private to that module.
const T0: u64 = 1000;

fn new_committing_game() -> Game {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    GameTrait::new(1, player_a(), player_b(), board_size, T0)
}

fn new_playing_game() -> Game {
    let mut game = new_committing_game();
    game.commit_root(player_a(), 0x111, T0);
    game.commit_root(player_b(), 0x222, T0);
    game
}

// ===============================
// Game::new() Tests
// ===============================

#[test]
fn test_new_game_6x6() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let game = GameTrait::new(1, player_a(), player_b(), board_size, T0);
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
    assert!(game.last_action_at == T0, "last_action_at should be initialized to now");
}

#[test]
fn test_new_game_8x8() {
    let board_size = BoardSize::Smaller(SmallerBoardSize::EightByEight);
    let game = GameTrait::new(2, player_a(), player_b(), board_size, T0);
    assert!(game.board_size.size() == 8, "Board size should be 8");
}

#[test]
fn test_new_game_10x10() {
    let game = GameTrait::new(3, player_a(), player_b(), BoardSize::Standard, T0);
    assert!(game.board_size.size() == 10, "Board size should be 10");
}

#[test]
fn test_new_game_12x12() {
    let board_size = BoardSize::Larger(LargerBoardSize::TwelveByTwelve);
    let game = GameTrait::new(4, player_a(), player_b(), board_size, T0);
    assert!(game.board_size.size() == 12, "Board size should be 12");
}

#[test]
fn test_new_game_14x14() {
    let board_size = BoardSize::Larger(LargerBoardSize::FourteenByFourteen);
    let game = GameTrait::new(5, player_a(), player_b(), board_size, T0);
    assert!(game.board_size.size() == 14, "Board size should be 14");
}

#[test]
fn test_new_game_20x20() {
    let board_size = BoardSize::Larger(LargerBoardSize::TwentyByTwenty);
    let game = GameTrait::new(6, player_a(), player_b(), board_size, T0);
    assert!(game.board_size.size() == 20, "Board size should be 20");
}

// ===============================
// Game::commit_root() Tests
// ===============================

#[test]
fn test_commit_root_player_a() {
    let mut game = new_committing_game();
    let root: felt252 = 0x123456;

    game.commit_root(player_a(), root, T0);

    assert!(game.player_a_root.is_some(), "Player A root should be set");
    assert!(game.player_a_root.unwrap() == root, "Player A root should match");
    assert!(game.player_b_root.is_none(), "Player B root should still be None");
    assert!(game.attacking_player.is_none(), "Attacking player should still be None");
}

#[test]
fn test_commit_root_player_b() {
    let mut game = new_committing_game();
    let root: felt252 = 0xabcdef;

    game.commit_root(player_b(), root, T0);

    assert!(game.player_b_root.is_some(), "Player B root should be set");
    assert!(game.player_b_root.unwrap() == root, "Player B root should match");
    assert!(game.player_a_root.is_none(), "Player A root should still be None");
    assert!(game.attacking_player.is_none(), "Attacking player should still be None");
}

#[test]
fn test_commit_root_both_players() {
    let mut game = new_committing_game();
    let root_a: felt252 = 0x111111;
    let root_b: felt252 = 0x222222;

    game.commit_root(player_a(), root_a, T0);
    game.commit_root(player_b(), root_b, T0 + 5);

    assert!(game.player_a_root.is_some(), "Player A root should be set");
    assert!(game.player_b_root.is_some(), "Player B root should be set");
    assert!(game.attacking_player.is_some(), "Attacking player should be set");
    let attacker: ContractAddress = game.attacking_player.unwrap();
    assert!(attacker == player_a(), "Player A should attack first");
    assert!(
        game.last_action_at == T0 + 5, "last_action_at should be bumped on the second commit only",
    );
}

#[test]
fn test_commit_root_first_does_not_bump_last_action_at() {
    // Anchored at game creation; first commit must not reset the deadline.
    let mut game = new_committing_game();
    game.commit_root(player_a(), 0x111, T0 + 50);

    assert!(
        game.last_action_at == T0,
        "last_action_at must not move on the first commit (parallel-action phase)",
    );
}

#[test]
fn test_commit_root_both_players_reverse_order() {
    let mut game = new_committing_game();
    let root_a: felt252 = 0x111111;
    let root_b: felt252 = 0x222222;

    game.commit_root(player_b(), root_b, T0);
    game.commit_root(player_a(), root_a, T0);

    assert!(game.attacking_player.is_some(), "Attacking player should be set");
    let attacker: ContractAddress = game.attacking_player.unwrap();
    assert!(attacker == player_a(), "Player A should attack first");
}

// ===============================
// Integration Tests
// ===============================

#[test]
fn test_full_game_flow_basic() {
    let mut game = new_committing_game();

    // Both players commit
    game.commit_root(player_a(), 0x111, T0);
    game.commit_root(player_b(), 0x222, T0);

    assert!(game.attacking_player.is_some(), "Game should be ready to start");
    let attacker: ContractAddress = game.attacking_player.unwrap();
    assert!(attacker == player_a(), "Player A goes first");
}

// ===============================
// check_timeout — Committing phase
// ===============================

#[test]
fn test_check_timeout_committing_not_yet_expired() {
    let game = new_committing_game();
    // Strict inequality: at exactly the deadline boundary, not yet expired.
    let now = T0 + COMMIT_TIMEOUT_SECS;
    assert!(game.check_timeout(now).is_none(), "Should not be expired at the deadline boundary");
}

#[test]
fn test_check_timeout_committing_neither_committed_returns_timeout() {
    let game = new_committing_game();
    let now = T0 + COMMIT_TIMEOUT_SECS + 1;
    let outcome = game.check_timeout(now).expect('expected Some');
    match outcome {
        Outcome::Timeout(some_loser) => {
            assert!(some_loser.is_none(), "Expected both players to lose.")
        },
        _ => panic!("Expected Outcome::Timeout when neither committed"),
    }
}

#[test]
fn test_check_timeout_committing_only_a_committed_blames_b() {
    let mut game = new_committing_game();
    game.commit_root(player_a(), 0x111, T0);
    let now = T0 + COMMIT_TIMEOUT_SECS + 1;
    let outcome = game.check_timeout(now).expect('expected Some');
    match outcome {
        Outcome::Timeout(loser) => assert_eq!(loser, Some(player_b()), "B should be blamed"),
        _ => panic!("Expected Outcome::Timeout(B)"),
    }
}

#[test]
fn test_check_timeout_committing_only_b_committed_blames_a() {
    let mut game = new_committing_game();
    game.commit_root(player_b(), 0x222, T0);
    let now = T0 + COMMIT_TIMEOUT_SECS + 1;
    let outcome = game.check_timeout(now).expect('expected Some');
    match outcome {
        Outcome::Timeout(loser) => assert_eq!(loser, Some(player_a()), "A should be blamed"),
        _ => panic!("Expected Outcome::Timeout(A)"),
    }
}

// ===============================
// check_timeout — Playing phase
// ===============================

#[test]
fn test_check_timeout_playing_not_yet_expired() {
    let game = new_playing_game();
    // Anchor for Playing is the second commit (T0 here). Boundary not expired.
    let now = T0 + TURN_TIMEOUT_SECS;
    assert!(game.check_timeout(now).is_none(), "Should not be expired at the deadline boundary");
}

#[test]
fn test_check_timeout_playing_attacker_inactive_blamed() {
    // Attacker (A) hasn't fired yet → A is the inactive party.
    let game = new_playing_game();
    let now = T0 + TURN_TIMEOUT_SECS + 1;
    let outcome = game.check_timeout(now).expect('expected Some');
    match outcome {
        Outcome::Timeout(loser) => assert_eq!(
            loser, Some(player_a()), "A should be blamed (attacker)",
        ),
        _ => panic!("Expected Outcome::Timeout(A)"),
    }
}

#[test]
fn test_check_timeout_playing_defender_inactive_blamed() {
    // Attacker (A) fired → B owes a defense → B is the inactive party.
    let mut game = new_playing_game();
    game.register_attack(player_a(), 0, 0, T0 + 10);
    let now = T0 + 10 + TURN_TIMEOUT_SECS + 1;
    let outcome = game.check_timeout(now).expect('expected Some');
    match outcome {
        Outcome::Timeout(loser) => assert_eq!(
            loser, Some(player_b()), "B should be blamed (defender)",
        ),
        _ => panic!("Expected Outcome::Timeout(B)"),
    }
}

// ===============================
// check_timeout — Revealing phase
// ===============================

/// Helper: bring a Game into Revealing phase by setting outcome_before_reveal directly.
/// This avoids the need to drive a full attack/defend sequence in unit tests.
fn drop_into_revealing(ref game: Game, anchor: u64) {
    use crate::types::OutcomeBeforeReveal;
    game.outcome_before_reveal = Some(OutcomeBeforeReveal::Fair(player_a()));
    game.last_action_at = anchor;
}

#[test]
fn test_check_timeout_revealing_not_yet_expired() {
    let mut game = new_playing_game();
    drop_into_revealing(ref game, T0);
    let now = T0 + REVEAL_TIMEOUT_SECS;
    assert!(game.check_timeout(now).is_none(), "Should not be expired at the deadline boundary");
}

#[test]
fn test_check_timeout_revealing_neither_revealed_returns_timeout() {
    let mut game = new_playing_game();
    drop_into_revealing(ref game, T0);
    let now = T0 + REVEAL_TIMEOUT_SECS + 1;
    let outcome = game.check_timeout(now).expect('expected Some');
    match outcome {
        Outcome::Timeout(some_loser) => {
            assert!(some_loser.is_none(), "Expected both players to be timed out.")
        },
        _ => panic!("Expected Outcome::Timeout when neither revealed"),
    }
}

#[test]
fn test_check_timeout_revealing_only_a_revealed_blames_b() {
    use crate::types::RevealStatus;
    let mut game = new_playing_game();
    drop_into_revealing(ref game, T0);
    game.player_a_reveal_status = Some(RevealStatus::Real);
    let now = T0 + REVEAL_TIMEOUT_SECS + 1;
    let outcome = game.check_timeout(now).expect('expected Some');
    match outcome {
        Outcome::Timeout(loser) => assert_eq!(loser, Some(player_b()), "B should be blamed"),
        _ => panic!("Expected Outcome::Timeout(B)"),
    }
}

#[test]
fn test_check_timeout_revealing_only_b_revealed_blames_a() {
    use crate::types::RevealStatus;
    let mut game = new_playing_game();
    drop_into_revealing(ref game, T0);
    game.player_b_reveal_status = Some(RevealStatus::Real);
    let now = T0 + REVEAL_TIMEOUT_SECS + 1;
    let outcome = game.check_timeout(now).expect('expected Some');
    match outcome {
        Outcome::Timeout(loser) => assert_eq!(loser, Some(player_a()), "A should be blamed"),
        _ => panic!("Expected Outcome::Timeout(A)"),
    }
}
