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
use starkwaves::types::{
    BoardSize, BoardSizeTrait, FireStatus, Orientation, Outcome, Ship, ShipKind, ShipKindTrait,
    SmallerBoardSize, create_board,
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
) -> (Array<u8>, felt252, Array<u8>, felt252) {
    let board_a = create_board(ships_a, 6);
    let board_b = create_board(ships_b, 6);

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

    let turn = |player: ContractAddress, x: u8, y: u8, hit_kind: Option<ShipKind>| {
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
        let status = hit_kind
            .map_or(
                FireStatus::Miss(pedersen(0, salt)),
                |kind| FireStatus::Hit((kind, pedersen(kind.id().into(), salt))),
            );
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
                                ship_kind: hit_kind,
                            },
                        ),
                    ),
                ],
            );
    };

    turn(player_a(), 0, 0, Some(ShipKind::Cruiser)); // Hit
    turn(player_b(), 0, 0, Some(ShipKind::Cruiser)); // Hit

    turn(player_a(), 4, 4, None); // Miss
    turn(player_b(), 4, 4, None); // Miss

    turn(player_a(), 0, 1, Some(ShipKind::Cruiser)); // Hit
    turn(player_b(), 0, 1, Some(ShipKind::Cruiser)); // Hit

    turn(player_a(), 0, 2, Some(ShipKind::Cruiser)); // Hit
    turn(player_b(), 0, 2, Some(ShipKind::Cruiser)); // Hit

    turn(player_a(), 1, 0, Some(ShipKind::Destroyer)); // Hit
    turn(player_b(), 1, 0, Some(ShipKind::Destroyer)); // Hit

    let mut spy = spy_events();

    turn(player_a(), 1, 1, Some(ShipKind::Destroyer)); // Hit + Reveal

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
