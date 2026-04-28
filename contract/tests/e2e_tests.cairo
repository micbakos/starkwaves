use core::pedersen::pedersen;
use merkle::{compute_merkle_root, generate_proof};
use snforge_std::{
    ContractClassTrait, DeclareResultTrait, EventSpyAssertionsTrait, declare, spy_events,
    start_cheat_caller_address,
};
use starknet::{ContractAddress, SyscallResultTrait};
use starkwaves::events::{AttackEvent, AttackResultEvent, GameOverEvent, GameRevealRequestEvent};
use starkwaves::starkwaves::Starkwaves::Event;
use starkwaves::starkwaves::{IStarkwavesDispatcher, IStarkwavesDispatcherTrait};
use starkwaves::types::board::{hulls_to_merkle_leaves, ships_to_hulls};
use starkwaves::types::{
    BoardSize, BoardSizeTrait, FireStatus, Orientation, Outcome, Ship, ShipKind, SmallerBoardSize,
};

// ===============================
// Helpers
// ===============================

fn player_a() -> ContractAddress {
    0x1.try_into().unwrap()
}

fn player_b() -> ContractAddress {
    0x2.try_into().unwrap()
}

fn owner() -> ContractAddress {
    0x999.try_into().unwrap()
}

fn deploy_starkwaves() -> ContractAddress {
    let contract = declare("Starkwaves").unwrap_syscall().contract_class();
    let (contract_address, _) = contract.deploy(@array![owner().into()]).unwrap_syscall();
    contract_address
}

fn create_6x6_ships() -> Array<Ship> {
    array![
        Ship { kind: ShipKind::Cruiser, x: 0, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 1, y: 0, orientation: Orientation::Horizontal },
    ]
}

/// Alternative ship placement for fake reveals.
fn create_6x6_alt_ships() -> Array<Ship> {
    array![
        Ship { kind: ShipKind::Cruiser, x: 3, y: 0, orientation: Orientation::Horizontal },
        Ship { kind: ShipKind::Destroyer, x: 4, y: 0, orientation: Orientation::Horizontal },
    ]
}

fn start_game(
    dispatcher: IStarkwavesDispatcher,
    contract_address: ContractAddress,
    player_a: ContractAddress,
    player_b: ContractAddress,
    board_size: BoardSize,
) -> felt252 {
    start_cheat_caller_address(contract_address, player_b);
    let result = dispatcher.request_start_game(board_size);
    assert!(result.is_none(), "First player should enter lobby");

    start_cheat_caller_address(contract_address, player_a);
    let game_id = dispatcher.request_start_game(board_size);
    game_id.expect('Game should start')
}

fn place(
    dispatcher: IStarkwavesDispatcher,
    contract_address: ContractAddress,
    game_id: felt252,
    ships_a: Span<Ship>,
    ships_b: Span<Ship>,
) -> (Array<bool>, felt252, Array<bool>, felt252) {
    let size = BoardSize::Smaller(SmallerBoardSize::SixBySix);
    let mut hulls_a = ships_to_hulls(ships_a, @size);
    let mut hulls_b = ships_to_hulls(ships_b, @size);

    let board_a = hulls_to_merkle_leaves(ref hulls_a, @size);
    let board_b = hulls_to_merkle_leaves(ref hulls_b, @size);

    let salt_a: felt252 = 12345;
    let salt_b: felt252 = 67890;

    let root_a = compute_merkle_root(board_a.clone(), salt_a);
    let root_b = compute_merkle_root(board_b.clone(), salt_b);

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.commit_board(root_a, game_id);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.commit_board(root_b, game_id);

    (board_a, salt_a, board_b, salt_b)
}

#[test]
#[fork("SEPOLIA_FORK")]
fn test_e2e_full_game_lifecycle() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    let game_id = start_game(dispatcher, contract_address, player_a(), player_b(), board_size);
    assert!(game_id == 1, "First game should have ID 1");
    let board_size = board_size.size();

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();

    let (board_a, salt_a, board_b, salt_b) = place(
        dispatcher, contract_address, game_id, ships_a.span(), ships_b.span(),
    );
    let board_a_span = board_a.span();
    let board_b_span = board_b.span();

    let turn = |
        player: ContractAddress, x: u8, y: u8, hit: bool, destroyed_kind: Option<ShipKind>,
    | {
        let (board, salt) = if (player == player_a()) {
            (board_b_span, salt_b)
        } else {
            (board_a_span, salt_a)
        };
        let opponent = if (player == player_a()) {
            player_b()
        } else {
            player_a()
        };

        let mut spy = spy_events();
        print!("Player {:?} attack at [{}, {}]", player, x, y);
        start_cheat_caller_address(contract_address, player);
        dispatcher.attack(game_id, x, y);

        spy
            .assert_emitted(
                @array![
                    (
                        contract_address,
                        Event::Attack(AttackEvent { game_id, player: player, x: x, y: y }),
                    ),
                ],
            );

        let proof = generate_proof(board, salt, (x * board_size + y).into());
        let status = if hit {
            FireStatus::Hit((destroyed_kind, pedersen(true.into(), salt)))
        } else {
            FireStatus::Miss(pedersen(false.into(), salt))
        };
        println!(" => {}", status);
        start_cheat_caller_address(contract_address, opponent);
        dispatcher.defend(game_id, status, proof);

        spy
            .assert_emitted(
                @array![
                    (
                        contract_address,
                        Event::AttackResult(
                            AttackResultEvent {
                                game_id,
                                attacker: player,
                                defender: opponent,
                                x,
                                y,
                                hit,
                                destroyed_ship_kind: destroyed_kind,
                            },
                        ),
                    ),
                ],
            );
    };

    turn(player_a(), 0, 0, true, None); // Hit
    turn(player_b(), 0, 0, true, None); // Hit

    turn(player_a(), 4, 4, false, None); // Miss
    turn(player_b(), 4, 4, false, None); // Miss

    turn(player_a(), 0, 1, true, None); // Hit
    turn(player_b(), 0, 1, true, None); // Hit

    turn(player_a(), 0, 2, true, Some(ShipKind::Cruiser)); // Hit + Destroy
    turn(player_b(), 0, 2, true, Some(ShipKind::Cruiser)); // Hit + Destroy

    turn(player_a(), 1, 0, true, None); // Hit
    turn(player_b(), 1, 0, true, None); // Hit

    let mut spy = spy_events();

    turn(player_a(), 1, 1, true, Some(ShipKind::Destroyer)); // Hit + Destroy + Reveal

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

    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, ships_a, salt_a);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, ships_b, salt_b);

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

// ===============================
// Cheating Tests - Invalid Proofs
// ===============================

/// Defender claims miss when the cell has a ship.
/// Proof verification fails → FailedToProvideProof(defender).
#[test]
#[fork("SEPOLIA_FORK")]
fn test_e2e_defender_claims_miss_on_hit() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    let game_id = start_game(dispatcher, contract_address, player_a(), player_b(), board_size);

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let (_board_a, salt_a, board_b, salt_b) = place(
        dispatcher, contract_address, game_id, ships_a.span(), ships_b.span(),
    );

    // Player A attacks (0,0) — Cruiser cell on B's board
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 0, 0);

    // Player B lies: claims miss on a ship cell
    let proof = generate_proof(board_b.span(), salt_b, 0);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, FireStatus::Miss(pedersen(false.into(), salt_b)), proof);
    // Verification fails: pedersen(false, salt) != leaf pedersen(true, salt)

    // Both reveal honestly
    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, ships_a, salt_a);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, ships_b, salt_b);

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

/// Defender claims hit when the cell is water.
/// Proof verification fails → FailedToProvideProof(defender).
#[test]
#[fork("SEPOLIA_FORK")]
fn test_e2e_defender_claims_hit_on_miss() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    let game_id = start_game(dispatcher, contract_address, player_a(), player_b(), board_size);

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let (_board_a, salt_a, board_b, salt_b) = place(
        dispatcher, contract_address, game_id, ships_a.span(), ships_b.span(),
    );

    // Player A attacks (5,5) — water cell on B's board
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.attack(game_id, 5, 5);

    // Player B lies: claims hit on a water cell
    let offset: u32 = 5 * 6 + 5;
    let proof = generate_proof(board_b.span(), salt_b, offset);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.defend(game_id, FireStatus::Hit((None, pedersen(true.into(), salt_b))), proof);
    // Verification fails: pedersen(true, salt) != leaf pedersen(false, salt)

    // Both reveal honestly
    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, ships_a, salt_a);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, ships_b, salt_b);

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

// ===============================
// Cheating Tests - Destruction Claims
// ===============================

/// Defender claims the wrong ship was destroyed (Destroyer instead of Cruiser).
/// Destruction hash won't match at reveal → RevealStatus::Fake.
#[test]
#[fork("SEPOLIA_FORK")]
fn test_e2e_false_destruction_claim() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    let game_id = start_game(dispatcher, contract_address, player_a(), player_b(), board_size);
    let size = board_size.size();

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let (board_a, salt_a, board_b, salt_b) = place(
        dispatcher, contract_address, game_id, ships_a.span(), ships_b.span(),
    );
    let board_a_span = board_a.span();
    let board_b_span = board_b.span();

    let play = |attacker: ContractAddress, x: u8, y: u8, hit: bool, destroyed: Option<ShipKind>| {
        let (board, salt) = if attacker == player_a() {
            (board_b_span, salt_b)
        } else {
            (board_a_span, salt_a)
        };
        let defender = if attacker == player_a() {
            player_b()
        } else {
            player_a()
        };

        start_cheat_caller_address(contract_address, attacker);
        dispatcher.attack(game_id, x, y);

        let proof = generate_proof(board, salt, (x * size + y).into());
        let status = if hit {
            FireStatus::Hit((destroyed, pedersen(true.into(), salt)))
        } else {
            FireStatus::Miss(pedersen(false.into(), salt))
        };
        start_cheat_caller_address(contract_address, defender);
        dispatcher.defend(game_id, status, proof);
    };

    // A systematically sinks B's ships; B always misses on A's board
    play(player_a(), 0, 0, true, None); // Hit B's Cruiser segment 1
    play(player_b(), 5, 5, false, None); // Miss

    play(player_a(), 0, 1, true, None); // Hit B's Cruiser segment 2
    play(player_b(), 5, 4, false, None); // Miss

    // A destroys B's Cruiser — B LIES: claims Destroyer instead of Cruiser
    play(player_a(), 0, 2, true, Some(ShipKind::Destroyer)); // ← CHEAT
    play(player_b(), 5, 3, false, None); // Miss

    play(player_a(), 1, 0, true, None); // Hit B's Destroyer segment 1
    play(player_b(), 5, 2, false, None); // Miss

    play(player_a(), 1, 1, true, Some(ShipKind::Destroyer)); // Destroy B's Destroyer → game over

    // Reveal: B's destruction hash has pedersen(pedersen(0, Destroyer), Destroyer)
    // but replay computes pedersen(pedersen(0, Cruiser), Destroyer) → mismatch → Fake
    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, ships_a, salt_a);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, ships_b, salt_b);

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

/// Defender omits a destruction claim when the ship is actually sunk.
/// Destruction hash won't match at reveal → RevealStatus::Fake.
#[test]
#[fork("SEPOLIA_FORK")]
fn test_e2e_omitted_destruction_claim() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    let game_id = start_game(dispatcher, contract_address, player_a(), player_b(), board_size);
    let size = board_size.size();

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let (board_a, salt_a, board_b, salt_b) = place(
        dispatcher, contract_address, game_id, ships_a.span(), ships_b.span(),
    );
    let board_a_span = board_a.span();
    let board_b_span = board_b.span();

    let play = |attacker: ContractAddress, x: u8, y: u8, hit: bool, destroyed: Option<ShipKind>| {
        let (board, salt) = if attacker == player_a() {
            (board_b_span, salt_b)
        } else {
            (board_a_span, salt_a)
        };
        let defender = if attacker == player_a() {
            player_b()
        } else {
            player_a()
        };

        start_cheat_caller_address(contract_address, attacker);
        dispatcher.attack(game_id, x, y);

        let proof = generate_proof(board, salt, (x * size + y).into());
        let status = if hit {
            FireStatus::Hit((destroyed, pedersen(true.into(), salt)))
        } else {
            FireStatus::Miss(pedersen(false.into(), salt))
        };
        start_cheat_caller_address(contract_address, defender);
        dispatcher.defend(game_id, status, proof);
    };

    // A systematically sinks B's ships; B always misses
    play(player_a(), 0, 0, true, None); // Hit B's Cruiser segment 1
    play(player_b(), 5, 5, false, None); // Miss

    play(player_a(), 0, 1, true, None); // Hit B's Cruiser segment 2
    play(player_b(), 5, 4, false, None); // Miss

    // A destroys B's Cruiser — B OMITS the destruction claim
    play(player_a(), 0, 2, true, None); // ← CHEAT: should be Some(ShipKind::Cruiser)
    play(player_b(), 5, 3, false, None); // Miss

    play(player_a(), 1, 0, true, None); // Hit B's Destroyer segment 1
    play(player_b(), 5, 2, false, None); // Miss

    play(player_a(), 1, 1, true, Some(ShipKind::Destroyer)); // Destroy B's Destroyer → game over

    // Reveal: B's hash has only pedersen(0, Destroyer)
    // but replay computes pedersen(pedersen(0, Cruiser), Destroyer) → mismatch → Fake
    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, ships_a, salt_a);
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, ships_b, salt_b);

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

// ===============================
// Cheating Tests - Fake Reveals
// ===============================

/// Player reveals with different ships than originally committed.
/// Root mismatch → RevealStatus::Fake.
#[test]
#[fork("SEPOLIA_FORK")]
fn test_e2e_fake_reveal() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    let game_id = start_game(dispatcher, contract_address, player_a(), player_b(), board_size);
    let size = board_size.size();

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let (board_a, salt_a, board_b, salt_b) = place(
        dispatcher, contract_address, game_id, ships_a.span(), ships_b.span(),
    );
    let board_a_span = board_a.span();
    let board_b_span = board_b.span();

    let play = |attacker: ContractAddress, x: u8, y: u8, hit: bool, destroyed: Option<ShipKind>| {
        let (board, salt) = if attacker == player_a() {
            (board_b_span, salt_b)
        } else {
            (board_a_span, salt_a)
        };
        let defender = if attacker == player_a() {
            player_b()
        } else {
            player_a()
        };

        start_cheat_caller_address(contract_address, attacker);
        dispatcher.attack(game_id, x, y);

        let proof = generate_proof(board, salt, (x * size + y).into());
        let status = if hit {
            FireStatus::Hit((destroyed, pedersen(true.into(), salt)))
        } else {
            FireStatus::Miss(pedersen(false.into(), salt))
        };
        start_cheat_caller_address(contract_address, defender);
        dispatcher.defend(game_id, status, proof);
    };

    // Play an honest full game — A wins
    play(player_a(), 0, 0, true, None);
    play(player_b(), 0, 0, true, None);

    play(player_a(), 0, 1, true, None);
    play(player_b(), 0, 1, true, None);

    play(player_a(), 0, 2, true, Some(ShipKind::Cruiser));
    play(player_b(), 0, 2, true, Some(ShipKind::Cruiser));

    play(player_a(), 1, 0, true, None);
    play(player_b(), 1, 0, true, None);

    play(player_a(), 1, 1, true, Some(ShipKind::Destroyer)); // Game over

    // A reveals honestly, B reveals with different ship positions
    let mut spy = spy_events();
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, ships_a, salt_a);

    let fake_ships_b = create_6x6_alt_ships();
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, fake_ships_b, salt_b);

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

/// Both players reveal with fake boards → Outcome::Null.
#[test]
#[fork("SEPOLIA_FORK")]
fn test_e2e_both_cheat_reveal() {
    let contract_address = deploy_starkwaves();
    let dispatcher = IStarkwavesDispatcher { contract_address };
    let board_size = BoardSize::Smaller(SmallerBoardSize::SixBySix);

    let game_id = start_game(dispatcher, contract_address, player_a(), player_b(), board_size);
    let size = board_size.size();

    let ships_a = create_6x6_ships();
    let ships_b = create_6x6_ships();
    let (board_a, salt_a, board_b, salt_b) = place(
        dispatcher, contract_address, game_id, ships_a.span(), ships_b.span(),
    );
    let board_a_span = board_a.span();
    let board_b_span = board_b.span();

    let play = |attacker: ContractAddress, x: u8, y: u8, hit: bool, destroyed: Option<ShipKind>| {
        let (board, salt) = if attacker == player_a() {
            (board_b_span, salt_b)
        } else {
            (board_a_span, salt_a)
        };
        let defender = if attacker == player_a() {
            player_b()
        } else {
            player_a()
        };

        start_cheat_caller_address(contract_address, attacker);
        dispatcher.attack(game_id, x, y);

        let proof = generate_proof(board, salt, (x * size + y).into());
        let status = if hit {
            FireStatus::Hit((destroyed, pedersen(true.into(), salt)))
        } else {
            FireStatus::Miss(pedersen(false.into(), salt))
        };
        start_cheat_caller_address(contract_address, defender);
        dispatcher.defend(game_id, status, proof);
    };

    // Play an honest full game — A wins
    play(player_a(), 0, 0, true, None);
    play(player_b(), 0, 0, true, None);

    play(player_a(), 0, 1, true, None);
    play(player_b(), 0, 1, true, None);

    play(player_a(), 0, 2, true, Some(ShipKind::Cruiser));
    play(player_b(), 0, 2, true, Some(ShipKind::Cruiser));

    play(player_a(), 1, 0, true, None);
    play(player_b(), 1, 0, true, None);

    play(player_a(), 1, 1, true, Some(ShipKind::Destroyer)); // Game over

    // Both reveal with fake ships
    let mut spy = spy_events();
    let fake_ships_a = create_6x6_alt_ships();
    start_cheat_caller_address(contract_address, player_a());
    dispatcher.reveal(game_id, fake_ships_a, salt_a);

    let fake_ships_b = create_6x6_alt_ships();
    start_cheat_caller_address(contract_address, player_b());
    dispatcher.reveal(game_id, fake_ships_b, salt_b);

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
