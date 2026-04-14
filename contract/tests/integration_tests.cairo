use core::pedersen::pedersen;
use merkle::{compute_merkle_root, generate_proof};
use snforge_std::{
    ContractClassTrait, DeclareResultTrait, EventSpyAssertionsTrait, declare, spy_events,
    start_cheat_caller_address,
};
use starknet::{ContractAddress, SyscallResultTrait};
use starkwaves::events::{
    AttackEvent, AttackResultEvent, GameOverEvent, GameRevealRequestEvent, GameStartedEvent,
    PlayerEnteredLobbyEvent, PlayersAssembledEvent,
};
use starkwaves::starkwaves::Starkwaves::Event;
use starkwaves::starkwaves::{IStarkwavesDispatcher, IStarkwavesDispatcherTrait};
use starkwaves::types::{
    BoardSize, FireStatus, Orientation, Outcome, Ship, ShipKind, ShipKindTrait, SmallerBoardSize,
    create_board,
};

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

fn owner() -> ContractAddress {
    0x999.try_into().unwrap()
}

fn deploy_starkwaves() -> ContractAddress {
    let contract = declare("Starkwaves").unwrap_syscall().contract_class();
    let (contract_address, _) = contract.deploy(@array![owner().into()]).unwrap_syscall();
    contract_address
}

/// Helper function to start a game using the lobby system.
/// player_a joins the lobby second and becomes the attacker (game.player_a).
/// player_b enters the lobby first and becomes the defender (game.player_b).
/// Returns the game_id with player_a as the current caller.
fn start_game_via_lobby(
    dispatcher: IStarkwavesDispatcher,
    contract_address: ContractAddress,
    player_a: ContractAddress,
    player_b: ContractAddress,
    board_size: BoardSize,
) -> felt252 {
    // player_b enters lobby first
    start_cheat_caller_address(contract_address, player_b);
    let result = dispatcher.request_start_game(board_size);
    assert!(result.is_none(), "First player should enter lobby");

    // player_a joins and starts game - becomes game.player_a
    start_cheat_caller_address(contract_address, player_a);
    let game_id = dispatcher.request_start_game(board_size);
    game_id.expect('Game should start')
}

fn create_6x6_ships() -> Array<Ship> {
    array![
        Ship { kind: ShipKind::Destroyer, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Cruiser, x: 2, y: 1, orientation: Orientation::Vertical },
    ]
}

fn get_cell_value(board: @Array<u8>, offset: u32) -> u8 {
    let cell_opt = board.get(offset);
    match cell_opt {
        Option::Some(boxed_value) => { *boxed_value.unbox() },
        Option::None => { panic!("Cell not found") },
    }
}

// ===============================
// Integration Tests - Lobby System
// ===============================

#[test]
fn test_integration_player_enters_lobby() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    start_cheat_caller_address(contract_address, player_a());
    let result = dispatcher.request_start_game(board_size);

    assert!(result.is_none(), "First player should enter lobby, not start game");
}

#[test]
fn test_integration_lobby_emits_event() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    let mut spy = spy_events();

    start_cheat_caller_address(contract_address, player_a());
    let _ = dispatcher.request_start_game(board_size);

    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::PlayerEntererLobby(
                        PlayerEnteredLobbyEvent { lobby: board_size, player: player_a() },
                    ),
                ),
            ],
        );
}

#[test]
fn test_integration_start_game() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    assert!(game_id == 1, "First game should have ID 1");
}

#[test]
fn test_integration_start_game_emits_event() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Player B enters lobby first
    start_cheat_caller_address(contract_address, player_b());
    let _ = dispatcher.request_start_game(board_size);

    let mut spy = spy_events();

    // Player A joins and starts game - becomes game.player_a
    start_cheat_caller_address(contract_address, player_a());
    let game_id = dispatcher.request_start_game(board_size).expect('Game should start');

    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::PlayersAssembled(
                        PlayersAssembledEvent {
                            player_a: player_a(), player_b: player_b(), game_id, board_size,
                        },
                    ),
                ),
            ],
        );
}

#[test]
#[should_panic(expected: "Cannot enter another lobby.")]
fn test_integration_player_cannot_enter_multiple_lobbies() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };

    start_cheat_caller_address(contract_address, player_a());
    let _ = dispatcher.request_start_game(BoardSize::Smaller(SmallerBoardSize::SixBySix));

    // Try to enter another lobby
    let _ = dispatcher.request_start_game(BoardSize::Smaller(SmallerBoardSize::EightByEight));
}

#[test]
#[should_panic(expected: "is already in another game.")]
fn test_integration_player_cannot_start_two_games() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Start first game
    start_game_via_lobby(dispatcher, contract_address, player_a(), player_b(), board_size);

    // Player A tries to join another lobby while in game
    start_cheat_caller_address(contract_address, player_a());
    let _ = dispatcher.request_start_game(board_size);
}

// ===============================
// Integration Tests - Commit Phase
// ===============================

#[test]
fn test_integration_both_players_commit() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Start game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    // Create boards
    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let board_a = create_board(ships_a.span(), 6);
    let board_b = create_board(ships_b.span(), 6);

    let salt_a: felt252 = 12345;
    let salt_b: felt252 = 67890;

    let root_a = compute_merkle_root(board_a, salt_a);
    let root_b = compute_merkle_root(board_b, salt_b);

    // Player A commits
    start_cheat_caller_address(contract_address, player_a());
    let mut spy = spy_events();
    dispatcher.commit_board(root_a, game_id);

    // Player B commits
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::GameStarted(
                        GameStartedEvent { game_id, attacker: player_a(), defender: player_b() },
                    ),
                ),
            ],
        );
}

#[test]
#[should_panic(expected: "has already committed")]
fn test_integration_player_cannot_commit_twice() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(0x123456, game_id);
    dispatcher.commit_board(0x654321, game_id); // Should panic
}

// ===============================
// Integration Tests - Attack/Defend
// ===============================

#[test]
fn test_integration_single_attack_defend_miss() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_b = create_6x6_ships();
    let board_b = create_board(ships_b.span(), 6);
    let salt_b: felt252 = 67890;
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(0x111111, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Player A attacks (5, 5) - water
    start_cheat_caller_address(contract_address, player_a());
    let mut spy = spy_events();
    dispatcher.attack(game_id, 5, 5);

    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::Attack(AttackEvent { game_id, player: player_a(), x: 5, y: 5 }),
                ),
            ],
        );

    // Player B defends with proof of miss
    let offset = 35; // 5 * 6 + 5
    let proof = generate_proof(board_b, salt_b, offset);
    let salted_status = pedersen(0, salt_b);
    let status = FireStatus::Miss(salted_status);

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status, proof);
}

#[test]
fn test_integration_single_attack_defend_hit() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_b = create_6x6_ships();
    let board_b = create_board(ships_b.span(), 6);
    let salt_b: felt252 = 67890;
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(0x111111, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Player A attacks (0, 0) - Destroyer
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 0, 0);

    // Player B defends with proof of hit
    let offset = 0;
    let proof = generate_proof(board_b, salt_b, offset);
    let cell_value = ShipKind::Destroyer.id();
    let salted_status = pedersen(cell_value.into(), salt_b);
    let status = FireStatus::Hit((ShipKind::Destroyer, salted_status));

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status, proof);
}

#[test]
#[should_panic(expected: "It is not player's")]
fn test_integration_wrong_player_attacks() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(0x111111, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(0x222222, game_id);

    // Player B tries to attack when it's Player A's turn
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.attack(game_id, 0, 0); // Should panic
}

#[test]
#[should_panic(expected: "out of bounds")]
fn test_integration_attack_out_of_bounds() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(0x111111, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(0x222222, game_id);

    // Player A attacks out of bounds
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 6, 6); // Should panic
}

// ===============================
// Integration Tests - Multiple Rounds
// ===============================

#[test]
fn test_integration_three_rounds_alternating() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let board_a = create_board(ships_a.span(), 6);
    let board_b = create_board(ships_b.span(), 6);

    let salt_a: felt252 = 12345;
    let salt_b: felt252 = 67890;

    let root_a = compute_merkle_root(board_a.clone(), salt_a);
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(root_a, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Round 1: Player A attacks, Player B defends
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 0, 0); // Hit Destroyer

    let proof = generate_proof(board_b.clone(), salt_b, 0);
    let cell_value = ShipKind::Destroyer.id();
    let salted_status = pedersen(cell_value.into(), salt_b);
    let status = FireStatus::Hit((ShipKind::Destroyer, salted_status));

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status, proof);

    // Round 2: Player B attacks, Player A defends
    dispatcher.attack(game_id, 5, 5); // Miss

    let proof2 = generate_proof(board_a.clone(), salt_a, 35);
    let salted_status2 = pedersen(0, salt_a);
    let status2 = FireStatus::Miss(salted_status2);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.defend(game_id, status2, proof2);

    // Round 3: Player A attacks again
    dispatcher.attack(game_id, 0, 1); // Hit Destroyer again

    let proof3 = generate_proof(board_b, salt_b, 1);
    let salted_status3 = pedersen(cell_value.into(), salt_b);
    let status3 = FireStatus::Hit((ShipKind::Destroyer, salted_status3));

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status3, proof3);
}

// ===============================
// Integration Tests - Complete Game
// ===============================

#[test]
fn test_integration_complete_game_player_a_wins() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let board_a = create_board(ships_a.span(), 6);
    let board_b = create_board(ships_b.span(), 6);

    let salt_a: felt252 = 12345;
    let salt_b: felt252 = 67890;

    let root_a = compute_merkle_root(board_a.clone(), salt_a);
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(root_a, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Player A needs 5 hits to win (Destroyer=2 + Cruiser=3)
    // Destroyer at (0,0) and (0,1)
    // Cruiser at (2,1), (3,1), (4,1)
    let hit_coords_b = array![(0_u8, 0_u8), (0, 1), (2, 1), (3, 1), (4, 1)];

    // Player B will miss these positions on Player A's board
    let miss_coords_a = array![(5_u8, 5_u8), (5, 4), (5, 3), (5, 2), (5, 1)];

    let mut i = 0;
    while i < hit_coords_b.len() {
        let (x, y) = *hit_coords_b.at(i);

        // Player A attacks Player B's board
        start_cheat_caller_address(contract_address, player_a());
        dispatcher.attack(game_id, x, y);

        // Calculate offset and get cell value
        let offset: u32 = x.into() * 6 + y.into();
        let cell_value = get_cell_value(@board_b, offset);
        let ship_kind = match cell_value {
            5 => ShipKind::Destroyer,
            3 => ShipKind::Cruiser,
            _ => panic!("Unexpected cell value"),
        };

        // Player B defends with proof of hit
        let proof = generate_proof(board_b.clone(), salt_b, offset);
        let salted_status = pedersen(cell_value.into(), salt_b);
        let status = FireStatus::Hit((ship_kind, salted_status));

        start_cheat_caller_address(contract_address, player_b());
        dispatcher.defend(game_id, status, proof);

        // Check if game is over
        if i == hit_coords_b.len() - 1 {
            // Last hit - game should be over
            break;
        }

        // Player B attacks Player A's board (miss)
        let (miss_x, miss_y) = *miss_coords_a.at(i);
        dispatcher.attack(game_id, miss_x, miss_y);

        // Player A defends with proof of miss
        let miss_offset: u32 = miss_x.into() * 6 + miss_y.into();
        let proof_a = generate_proof(board_a.clone(), salt_a, miss_offset);
        let salted_miss = pedersen(0, salt_a);
        let miss_status = FireStatus::Miss(salted_miss);

        start_cheat_caller_address(contract_address, player_a());
        dispatcher.defend(game_id, miss_status, proof_a);

        i += 1;
    };
    // Game should be over after Player A gets all 5 hits
}

#[test]
fn test_integration_mixed_hits_and_misses() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let board_a = create_board(ships_a.span(), 6);
    let board_b = create_board(ships_b.span(), 6);

    let salt_a: felt252 = 12345;
    let salt_b: felt252 = 67890;

    let root_a = compute_merkle_root(board_a.clone(), salt_a);
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(root_a, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Round 1: Player A misses

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 5, 5);

    let proof = generate_proof(board_b.clone(), salt_b, 35);
    let salted_status = pedersen(0, salt_b);
    let status = FireStatus::Miss(salted_status);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status, proof);

    // Round 2: Player B misses
    dispatcher.attack(game_id, 5, 4);

    let proof2 = generate_proof(board_a.clone(), salt_a, 34);
    let salted_status2 = pedersen(0, salt_a);
    let status2 = FireStatus::Miss(salted_status2);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.defend(game_id, status2, proof2);

    // Round 3: Player A hits
    dispatcher.attack(game_id, 0, 0);

    let proof3 = generate_proof(board_b.clone(), salt_b, 0);
    let cell_value = ShipKind::Destroyer.id();
    let salted_status3 = pedersen(cell_value.into(), salt_b);
    let status3 = FireStatus::Hit((ShipKind::Destroyer, salted_status3));

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status3, proof3);

    // Round 4: Player B hits
    dispatcher.attack(game_id, 2, 1);

    let proof4 = generate_proof(board_a, salt_a, 13);
    let cell_value_a = ShipKind::Cruiser.id();
    let salted_status4 = pedersen(cell_value_a.into(), salt_a);
    let status4 = FireStatus::Hit((ShipKind::Cruiser, salted_status4));

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.defend(game_id, status4, proof4);
}

#[test]
#[should_panic(expected: "The (0, 0) is already bombed")]
fn test_integration_cannot_attack_same_position_twice() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let board_a = create_board(ships_a.span(), 6);
    let board_b = create_board(ships_b.span(), 6);

    let salt_a: felt252 = 12345;
    let salt_b: felt252 = 67890;

    let root_a = compute_merkle_root(board_a.clone(), salt_a);
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(root_a, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Player A attacks (0, 0)
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 0, 0);

    let proof = generate_proof(board_b.clone(), salt_b, 0);
    let cell_value = ShipKind::Destroyer.id();
    let salted_status = pedersen(cell_value.into(), salt_b);
    let status = FireStatus::Hit((ShipKind::Destroyer, salted_status));

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status, proof);

    // Player B's turn
    dispatcher.attack(game_id, 1, 1);

    let proof2 = generate_proof(board_a, salt_a, 7);
    let salted_status2 = pedersen(0, salt_a);
    let status2 = FireStatus::Miss(salted_status2);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.defend(game_id, status2, proof2);

    // Player A tries to attack (0, 0) again
    dispatcher.attack(game_id, 0, 0); // Should panic
}

// ===============================
// Integration Tests - Board Sizes
// ===============================

#[test]
fn test_integration_different_board_sizes() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let player_d: ContractAddress = 0x4.try_into().unwrap();

    // Test 8x8 - Player A enters lobby, Player B joins
    let game_id_8 = start_game_via_lobby(
        dispatcher,
        contract_address,
        player_a(),
        player_b(),
        BoardSize::Smaller(SmallerBoardSize::EightByEight),
    );
    assert!(game_id_8 == 1, "First game should be ID 1");

    // Test 10x10 - Player C enters lobby, Player D joins (Standard board)
    let game_id_10 = start_game_via_lobby(
        dispatcher, contract_address, player_c(), player_d, BoardSize::Standard,
    );
    assert!(game_id_10 == 2, "Second game should be ID 2");
}

// ===============================
// Tests for Recent Changes
// ===============================

#[test]
fn test_players_assembled_event_on_game_creation() {
    // Test that PlayersAssembledEvent is emitted when game is created via lobby
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Player B enters lobby first
    start_cheat_caller_address(contract_address, player_b());
    let _ = dispatcher.request_start_game(board_size);

    let mut spy = spy_events();

    // Player A joins and starts game - becomes game.player_a
    start_cheat_caller_address(contract_address, player_a());
    let game_id = dispatcher.request_start_game(board_size).expect('Game should start');

    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::PlayersAssembled(
                        PlayersAssembledEvent {
                            game_id, player_a: player_a(), player_b: player_b(), board_size,
                        },
                    ),
                ),
            ],
        );
}

#[test]
fn test_game_started_event_only_after_both_commits() {
    // Test that GameStartedEvent is only emitted after BOTH players commit
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    // Player A commits - should NOT emit GameStartedEvent yet
    start_cheat_caller_address(contract_address, player_a());
    let mut spy = spy_events();
    dispatcher.commit_board(0x111111, game_id);

    // Verify no GameStartedEvent was emitted after first commit
    // Use a dummy GameStartedEvent to check it was NOT emitted
    spy
        .assert_not_emitted(
            @array![
                (
                    contract_address,
                    Event::GameStarted(
                        GameStartedEvent { game_id, attacker: player_a(), defender: player_b() },
                    ),
                ),
            ],
        );

    // Player B commits - NOW should emit GameStartedEvent
    spy = spy_events();
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(0x222222, game_id);

    // Should emit GameStartedEvent with attacker and defender
    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::GameStarted(
                        GameStartedEvent { game_id, attacker: player_a(), defender: player_b() },
                    ),
                ),
            ],
        );
}

#[test]
fn test_attack_result_event_on_miss_has_no_ship_kind() {
    // Test that AttackResultEvent is emitted with ship_kind: None on a miss
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_b = create_6x6_ships();
    let board_b = create_board(ships_b.span(), 6);
    let salt_b: felt252 = 67890;
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(0x111111, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Attack a miss position
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 5, 5); // Water

    let mut spy = spy_events();
    let proof = generate_proof(board_b.clone(), salt_b, 35);
    let salted_status = pedersen(0, salt_b);
    let status = FireStatus::Miss(salted_status);

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status, proof);

    // Verify AttackResultEvent was emitted with ship_kind: None
    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::AttackResult(
                        AttackResultEvent {
                            game_id,
                            attacker: player_a(),
                            defender: player_b(),
                            x: 5,
                            y: 5,
                            ship_kind: None,
                        },
                    ),
                ),
            ],
        );
}

#[test]
fn test_attack_result_event_emitted_on_actual_hit() {
    // Test that AttackResultEvent IS emitted with ship_kind: Some(_) on actual hits
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game with BOTH players having valid board roots
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let board_a = create_board(ships_a.span(), 6);
    let board_b = create_board(ships_b.span(), 6);
    let salt_a: felt252 = 12345;
    let salt_b: felt252 = 67890;

    let root_a = compute_merkle_root(board_a.clone(), salt_a);
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(root_a, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Player A attacks (0, 0) - Destroyer position on Player B's board
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 0, 0);

    // Player B defends with proof of hit
    let mut spy = spy_events();
    let proof = generate_proof(board_b, salt_b, 0);
    let cell_value = ShipKind::Destroyer.id();
    let salted_status = pedersen(cell_value.into(), salt_b);
    let status = FireStatus::Hit((ShipKind::Destroyer, salted_status));

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status, proof);

    // Should emit AttackResultEvent with ship_kind: Some(Destroyer)
    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::AttackResult(
                        AttackResultEvent {
                            game_id,
                            attacker: player_a(),
                            defender: player_b(),
                            x: 0,
                            y: 0,
                            ship_kind: Some(ShipKind::Destroyer),
                        },
                    ),
                ),
            ],
        );
}

#[test]
#[should_panic(expected: "is not playing in")]
fn test_non_player_cannot_commit() {
    // Test that a player not in the game cannot commit
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    // Player C (not in game) tries to commit
    start_cheat_caller_address(contract_address, player_c());
    dispatcher.commit_board(0x333333, game_id); // Should panic
}

#[test]
fn test_no_events_emitted_on_single_commit() {
    // Test that only PlayersAssembled is emitted on game creation,
    // and no game-start events until both commit
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    // Now commit only one player
    start_cheat_caller_address(contract_address, player_a());
    let mut spy = spy_events();
    dispatcher.commit_board(0x111111, game_id);

    // Check that GameStartedEvent was NOT emitted using assert_not_emitted
    spy
        .assert_not_emitted(
            @array![
                (
                    contract_address,
                    Event::GameStarted(
                        GameStartedEvent { game_id, attacker: player_a(), defender: player_b() },
                    ),
                ),
            ],
        );
}

#[test]
fn test_game_reveal_request_on_failed_proof() {
    // Test that GameRevealRequestEvent is emitted when proof verification fails
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_b = create_6x6_ships();
    let board_b = create_board(ships_b.span(), 6);
    let salt_b: felt252 = 67890;
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(0x111111, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Player A attacks
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 0, 0);

    // Player B provides WRONG proof (empty proof array)
    let mut spy = spy_events();
    let wrong_proof = array![];
    let salted_status = pedersen(ShipKind::Destroyer.id().into(), salt_b);
    let status = FireStatus::Hit((ShipKind::Destroyer, salted_status));

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status, wrong_proof);

    // Should emit GameRevealRequestEvent
    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::GameRevealRequest(
                        GameRevealRequestEvent {
                            game_id, player_a: player_a(), player_b: player_b(),
                        },
                    ),
                ),
            ],
        );
}

// ===============================
// Integration Tests - Reveal Phase
// ===============================

/// Helper function to play a complete game where Player A wins by hitting all ships.
/// Returns (game_id, board_a, board_b, salt_a, salt_b)
fn play_complete_game_player_a_wins(
    dispatcher: IStarkwavesDispatcher, contract_address: ContractAddress,
) -> (felt252, Array<u8>, Array<u8>, felt252, felt252) {
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let board_a = create_board(ships_a.span(), 6);
    let board_b = create_board(ships_b.span(), 6);

    let salt_a: felt252 = 12345;
    let salt_b: felt252 = 67890;

    let root_a = compute_merkle_root(board_a.clone(), salt_a);
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(root_a, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Player A needs 5 hits to win (Destroyer=2 + Cruiser=3)
    // Destroyer at (0,0) and (0,1)
    // Cruiser at (2,1), (3,1), (4,1)
    let hit_coords_b = array![(0_u8, 0_u8), (0, 1), (2, 1), (3, 1), (4, 1)];

    // Player B will miss these positions on Player A's board
    let miss_coords_a = array![(5_u8, 5_u8), (5, 4), (5, 3), (5, 2)];

    let mut i = 0;
    while i < hit_coords_b.len() {
        let (x, y) = *hit_coords_b.at(i);

        // Player A attacks Player B's board
        start_cheat_caller_address(contract_address, player_a());
        dispatcher.attack(game_id, x, y);

        // Calculate offset and get cell value
        let offset: u32 = x.into() * 6 + y.into();
        let cell_value = get_cell_value(@board_b, offset);
        let ship_kind = match cell_value {
            5 => ShipKind::Destroyer,
            3 => ShipKind::Cruiser,
            _ => panic!("Unexpected cell value"),
        };

        // Player B defends with proof of hit
        let proof = generate_proof(board_b.clone(), salt_b, offset);
        let salted_status = pedersen(cell_value.into(), salt_b);
        let status = FireStatus::Hit((ship_kind, salted_status));

        start_cheat_caller_address(contract_address, player_b());
        dispatcher.defend(game_id, status, proof);

        // Check if game is over (last hit)
        if i == hit_coords_b.len() - 1 {
            break;
        }

        // Player B attacks Player A's board (miss)
        let (miss_x, miss_y) = *miss_coords_a.at(i);
        dispatcher.attack(game_id, miss_x, miss_y);

        // Player A defends with proof of miss
        let miss_offset: u32 = miss_x.into() * 6 + miss_y.into();
        let proof_a = generate_proof(board_a.clone(), salt_a, miss_offset);
        let salted_miss = pedersen(0, salt_a);
        let miss_status = FireStatus::Miss(salted_miss);

        start_cheat_caller_address(contract_address, player_a());
        dispatcher.defend(game_id, miss_status, proof_a);

        i += 1;
    }

    (game_id, board_a, board_b, salt_a, salt_b)
}

#[test]
fn test_reveal_both_players_honest_fair_outcome() {
    // Complete game where Player A wins, both players reveal honestly
    // Expected: Fair(player_a) outcome
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };

    let (game_id, board_a, board_b, salt_a, salt_b) = play_complete_game_player_a_wins(
        dispatcher, contract_address,
    );

    // Player A reveals their board
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, board_a.clone(), salt_a);

    // Player B reveals their board - this should trigger GameOverEvent
    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, board_b.clone(), salt_b);

    // Should emit GameOverEvent with Fair outcome (Player A won legitimately)
    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::GameOver(
                        GameOverEvent {
                            game_id,
                            player_a: player_a(),
                            player_b: player_b(),
                            outcome: Outcome::Fair(player_a()),
                        },
                    ),
                ),
            ],
        );
}

#[test]
fn test_reveal_player_b_reveals_fake_board() {
    // Complete game where Player A wins, but Player B reveals a fake board
    // Expected: FailedToProvideProof(player_b) - Player A wins because B cheated
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };

    let (game_id, board_a, _board_b, salt_a, salt_b) = play_complete_game_player_a_wins(
        dispatcher, contract_address,
    );

    // Create a fake board for Player B (different ship positions)
    let fake_ships_b = array![
        Ship { kind: ShipKind::Destroyer, x: 5, y: 4, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Vertical },
    ];
    let fake_board_b = create_board(fake_ships_b.span(), 6);

    // Player A reveals honestly
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, board_a.clone(), salt_a);

    // Player B reveals fake board
    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, fake_board_b, salt_b);

    // Should emit GameOverEvent with FailedToProvideProof - A wins because B was dishonest
    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::GameOver(
                        GameOverEvent {
                            game_id,
                            player_a: player_a(),
                            player_b: player_b(),
                            outcome: Outcome::FailedToProvideProof(player_b()),
                        },
                    ),
                ),
            ],
        );
}

#[test]
fn test_reveal_player_a_reveals_fake_board() {
    // Complete game where Player A wins, but Player A reveals a fake board
    // Expected: FailedToProvideProof(player_a) - Player B wins because A cheated
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };

    let (game_id, _board_a, board_b, salt_a, salt_b) = play_complete_game_player_a_wins(
        dispatcher, contract_address,
    );

    // Create a fake board for Player A
    let fake_ships_a = array![
        Ship { kind: ShipKind::Destroyer, x: 5, y: 4, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Vertical },
    ];
    let fake_board_a = create_board(fake_ships_a.span(), 6);

    // Player A reveals fake board
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, fake_board_a, salt_a);

    // Player B reveals honestly
    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, board_b.clone(), salt_b);

    // Should emit GameOverEvent with FailedToProvideProof - B wins because A was dishonest
    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::GameOver(
                        GameOverEvent {
                            game_id,
                            player_a: player_a(),
                            player_b: player_b(),
                            outcome: Outcome::FailedToProvideProof(player_a()),
                        },
                    ),
                ),
            ],
        );
}

#[test]
fn test_reveal_both_players_reveal_fake_boards() {
    // Complete game where Player A wins, but BOTH players reveal fake boards
    // Expected: Null outcome (both cheated)
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };

    let (game_id, _board_a, _board_b, salt_a, salt_b) = play_complete_game_player_a_wins(
        dispatcher, contract_address,
    );

    // Create fake boards for both players
    let fake_ships = array![
        Ship { kind: ShipKind::Destroyer, x: 5, y: 4, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Vertical },
    ];
    let fake_board = create_board(fake_ships.span(), 6);

    // Player A reveals fake board
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, fake_board.clone(), salt_a);

    // Player B reveals fake board
    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, fake_board.clone(), salt_b);

    // Should emit GameOverEvent with Null outcome (both cheated)
    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::GameOver(
                        GameOverEvent {
                            game_id,
                            player_a: player_a(),
                            player_b: player_b(),
                            outcome: Outcome::Null,
                        },
                    ),
                ),
            ],
        );
}

#[test]
fn test_reveal_after_failed_proof_honest_reveals() {
    // Game ends because Player B failed to provide proof during defend
    // Both players reveal honestly
    // Expected: FailedToProvideProof(player_b) - Player A wins because B cheated during game
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let board_a = create_board(ships_a.span(), 6);
    let board_b = create_board(ships_b.span(), 6);

    let salt_a: felt252 = 12345;
    let salt_b: felt252 = 67890;

    let root_a = compute_merkle_root(board_a.clone(), salt_a);
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(root_a, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Player A attacks
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 0, 0);

    // Player B provides WRONG proof (triggers FailedToProvideProof)
    let wrong_proof = array![];
    let salted_status = pedersen(ShipKind::Destroyer.id().into(), salt_b);
    let status = FireStatus::Hit((ShipKind::Destroyer, salted_status));

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status, wrong_proof);

    // Now both reveal honestly
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, board_a.clone(), salt_a);

    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, board_b.clone(), salt_b);

    // Player A wins because B cheated during game (failed proof)
    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::GameOver(
                        GameOverEvent {
                            game_id,
                            player_a: player_a(),
                            player_b: player_b(),
                            outcome: Outcome::FailedToProvideProof(player_b()),
                        },
                    ),
                ),
            ],
        );
}

#[test]
fn test_reveal_after_failed_proof_cheater_also_reveals_fake() {
    // Game ends because Player B failed to provide proof during defend
    // Player B also reveals a fake board (double cheater)
    // Player A reveals honestly
    // Expected: FailedToProvideProof(player_b) - Player A wins
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let board_a = create_board(ships_a.span(), 6);
    let board_b = create_board(ships_b.span(), 6);

    let salt_a: felt252 = 12345;
    let salt_b: felt252 = 67890;

    let root_a = compute_merkle_root(board_a.clone(), salt_a);
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(root_a, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Player A attacks
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 0, 0);

    // Player B provides WRONG proof
    let wrong_proof = array![];
    let salted_status = pedersen(ShipKind::Destroyer.id().into(), salt_b);
    let status = FireStatus::Hit((ShipKind::Destroyer, salted_status));

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status, wrong_proof);

    // Player A reveals honestly
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, board_a.clone(), salt_a);

    // Player B reveals FAKE board (double cheater)
    let fake_ships = array![
        Ship { kind: ShipKind::Destroyer, x: 5, y: 4, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Vertical },
    ];
    let fake_board = create_board(fake_ships.span(), 6);

    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, fake_board, salt_b);

    // Player A wins because B cheated (both during game and on reveal)
    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::GameOver(
                        GameOverEvent {
                            game_id,
                            player_a: player_a(),
                            player_b: player_b(),
                            outcome: Outcome::FailedToProvideProof(player_b()),
                        },
                    ),
                ),
            ],
        );
}

#[test]
fn test_reveal_after_failed_proof_both_cheat_on_reveal() {
    // Game ends because Player B failed to provide proof during defend
    // BOTH players reveal fake boards
    // Expected: Null outcome (both are now cheaters)
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let board_a = create_board(ships_a.span(), 6);
    let board_b = create_board(ships_b.span(), 6);

    let salt_a: felt252 = 12345;
    let salt_b: felt252 = 67890;

    let root_a = compute_merkle_root(board_a.clone(), salt_a);
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(root_a, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Player A attacks
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 0, 0);

    // Player B provides WRONG proof
    let wrong_proof = array![];
    let salted_status = pedersen(ShipKind::Destroyer.id().into(), salt_b);
    let status = FireStatus::Hit((ShipKind::Destroyer, salted_status));

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status, wrong_proof);

    // Both reveal FAKE boards
    let fake_ships = array![
        Ship { kind: ShipKind::Destroyer, x: 5, y: 4, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Vertical },
    ];
    let fake_board = create_board(fake_ships.span(), 6);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, fake_board.clone(), salt_a);

    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, fake_board.clone(), salt_b);

    // Null outcome because both cheated (A on reveal, B during game and reveal)
    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::GameOver(
                        GameOverEvent {
                            game_id,
                            player_a: player_a(),
                            player_b: player_b(),
                            outcome: Outcome::Null,
                        },
                    ),
                ),
            ],
        );
}

#[test]
#[should_panic(expected: "has already revealed")]
fn test_reveal_player_cannot_reveal_twice() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };

    let (game_id, board_a, _board_b, salt_a, _salt_b) = play_complete_game_player_a_wins(
        dispatcher, contract_address,
    );

    // Player A reveals their board
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, board_a.clone(), salt_a);

    // Player A tries to reveal again - should panic
    dispatcher.reveal(game_id, board_a.clone(), salt_a);
}

#[test]
#[should_panic(expected: "game is not finished")]
fn test_reveal_fails_if_game_not_finished() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    // Setup game but don't finish it
    let game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let board_a = create_board(ships_a.span(), 6);
    let board_b = create_board(ships_b.span(), 6);

    let salt_a: felt252 = 12345;
    let salt_b: felt252 = 67890;

    let root_a = compute_merkle_root(board_a.clone(), salt_a);
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(root_a, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    // Only one attack/defend - game not finished yet
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 5, 5);

    let proof = generate_proof(board_b.clone(), salt_b, 35);
    let salted_status = pedersen(0, salt_b);
    let status = FireStatus::Miss(salted_status);

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, status, proof);

    // Try to reveal - should fail because game is not finished
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, board_a.clone(), salt_a);
}

#[test]
fn test_reveal_clears_game_from_storage() {
    // After both reveal, players should be able to start a new game
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };

    let (game_id, board_a, board_b, salt_a, salt_b) = play_complete_game_player_a_wins(
        dispatcher, contract_address,
    );

    // Both players reveal
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, board_a.clone(), salt_a);

    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, board_b.clone(), salt_b);

    // Now both players should be able to start a new game
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let new_game_id = start_game_via_lobby(
        dispatcher, contract_address, player_a(), player_b(), board_size,
    );

    assert!(new_game_id == 2, "Should be able to start a new game after reveal");
}

#[test]
fn test_reveal_order_does_not_matter() {
    // Test that Player B can reveal first, then Player A
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };

    let (game_id, board_a, board_b, salt_a, salt_b) = play_complete_game_player_a_wins(
        dispatcher, contract_address,
    );

    // Player B reveals first
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, board_b.clone(), salt_b);

    // Player A reveals second - should trigger GameOverEvent
    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, board_a.clone(), salt_a);

    // Should still emit Fair(player_a) since A won the game
    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::GameOver(
                        GameOverEvent {
                            game_id,
                            player_a: player_a(),
                            player_b: player_b(),
                            outcome: Outcome::Fair(player_a()),
                        },
                    ),
                ),
            ],
        );
}

#[test]
fn test_reveal_with_wrong_salt() {
    // Player reveals with correct board but wrong salt - should be considered fake
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };

    let (game_id, board_a, board_b, _salt_a, salt_b) = play_complete_game_player_a_wins(
        dispatcher, contract_address,
    );

    // Player A reveals with wrong salt
    let wrong_salt: felt252 = 99999;
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, board_a.clone(), wrong_salt);

    // Player B reveals correctly
    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, board_b.clone(), salt_b);

    // Player B wins because A revealed with wrong salt (considered dishonest)
    spy
        .assert_emitted(
            @array![
                (
                    contract_address,
                    Event::GameOver(
                        GameOverEvent {
                            game_id,
                            player_a: player_a(),
                            player_b: player_b(),
                            outcome: Outcome::FailedToProvideProof(player_a()),
                        },
                    ),
                ),
            ],
        );
}
