use crate::types::phase::TimeoutConfig;
use crate::types::{BoardSize, FireStatus, Lobbies, Lobby, Ship};

#[starknet::interface]
pub trait IStarkwaves<TContractState> {
    fn exit_lobby(ref self: TContractState, board_size: BoardSize);

    fn request_start_game(ref self: TContractState, board_size: BoardSize) -> Option<felt252>;

    fn commit_board(ref self: TContractState, root: felt252, game_id: felt252);

    fn attack(ref self: TContractState, game_id: felt252, x: u8, y: u8);

    fn defend(
        ref self: TContractState, game_id: felt252, status: FireStatus, proof: Array<felt252>,
    );

    fn reveal(ref self: TContractState, game_id: felt252, ships: Array<Ship>, salt: felt252);

    fn claim_timeout(ref self: TContractState, game_id: felt252);

    fn reset(ref self: TContractState);

    fn get_next_game_id(self: @TContractState) -> felt252;

    fn get_timeout_config(self: @TContractState) -> TimeoutConfig;

    fn get_lobbies(self: @TContractState) -> Lobbies;
}

#[starknet::contract]
pub mod Starkwaves {
    use core::num::traits::Zero;
    use openzeppelin_access::ownable::OwnableComponent;
    use starknet::event::EventEmitter;
    use starknet::storage::{
        Map, StorageMapReadAccess, StoragePathEntry, StoragePointerReadAccess,
        StoragePointerWriteAccess,
    };
    use starknet::{ContractAddress, get_block_timestamp, get_caller_address};
    use crate::events::{
        AttackEvent, AttackResultEvent, GameOverEvent, GameRevealRequestEvent, GameStartedEvent,
        PlayerEnteredLobbyEvent, PlayersAssembledEvent, ResetEvent,
    };
    use crate::game::{Game, GameTrait};
    use crate::types::{AllBoardSizesTrait, BoardSizeTrait, Outcome};
    use super::{*, BoardSize, FireStatus, Lobby, TimeoutConfig};

    component!(path: OwnableComponent, storage: ownable, event: OwnableEvent);
    #[abi(embed_v0)]
    impl OwnableImpl = OwnableComponent::OwnableImpl<ContractState>;
    impl OwnableInternalImpl = OwnableComponent::InternalImpl<ContractState>;

    #[storage]
    struct Storage {
        open_lobbies: Map<u8, ContractAddress>,
        next_game_id: felt252,
        open_games: Map<felt252, Game>,
        open_games_per_player: Map<ContractAddress, felt252>,
        // Storage for other components
        #[substorage(v0)]
        ownable: OwnableComponent::Storage,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        PlayerEntererLobby: PlayerEnteredLobbyEvent,
        PlayersAssembled: PlayersAssembledEvent,
        GameStarted: GameStartedEvent,
        Attack: AttackEvent,
        AttackResult: AttackResultEvent,
        GameRevealRequest: GameRevealRequestEvent,
        GameOver: GameOverEvent,
        Reset: ResetEvent,
        // Events from other components
        #[flat]
        OwnableEvent: OwnableComponent::Event,
    }

    #[constructor]
    fn constructor(ref self: ContractState, owner: ContractAddress) {
        self.ownable.initializer(owner);
        self.next_game_id.write(1);
    }

    #[abi(embed_v0)]
    impl StarkwavesImpl of super::IStarkwaves<ContractState> {
        fn exit_lobby(ref self: ContractState, board_size: BoardSize) {
            let player = get_caller_address();
            let size = board_size.size();
            let some_player = self.open_lobbies.entry(size).read();

            assert!(player == some_player, "Player {:?} is not in lobby {}", player, board_size);

            self.open_lobbies.entry(size).write(Zero::zero());
        }

        fn request_start_game(ref self: ContractState, board_size: BoardSize) -> Option<felt252> {
            let player = get_caller_address();

            let game_id = self.open_games_per_player.entry(player).read();
            assert!(game_id == 0, "Player {:?} is already in another game.", player);

            let all_board_sizes = AllBoardSizesTrait::all();
            for board_size in all_board_sizes {
                let size = board_size.size();
                let a_player = self.open_lobbies.entry(size).read();
                assert!(a_player != player, "Cannot enter another lobby.")
            }

            let size = board_size.size();
            let opponent = self.open_lobbies.entry(size).read();
            if opponent.is_zero() {
                // Enter lobby
                self.open_lobbies.entry(size).write(player);
                self.emit(PlayerEnteredLobbyEvent { lobby: board_size, player: player });

                None
            } else {
                self.open_lobbies.entry(size).write(Zero::zero());
                let game_id = self.start_game(opponent, board_size);

                Some(game_id)
            }
        }

        fn commit_board(ref self: ContractState, root: felt252, game_id: felt252) {
            let player = self.assert_player_in_game(game_id);

            let mut game = self.open_games.read(game_id);
            let now = get_block_timestamp();
            if self.settle_timeout(@game, now) {
                return;
            }

            game.commit_root(player, root, now);
            self.open_games.entry(game_id).write(game.clone());

            if let Some(attacking_player) = game.attacking_player {
                let defender = game.defender().expect('Defender should exist.');
                self
                    .emit(
                        GameStartedEvent {
                            game_id: game.id, attacker: attacking_player, defender: defender,
                        },
                    )
            }
        }

        fn attack(ref self: ContractState, game_id: felt252, x: u8, y: u8) {
            let player = self.assert_player_in_game(game_id);
            let mut game = self.open_games.read(game_id);

            let now = get_block_timestamp();
            if self.settle_timeout(@game, now) {
                return;
            }

            game.register_attack(player, x, y, now);

            self.open_games.entry(game_id).write(game);
            self.emit(AttackEvent { game_id, player, x, y })
        }

        fn defend(
            ref self: ContractState, game_id: felt252, status: FireStatus, proof: Array<felt252>,
        ) {
            let defender = self.assert_player_in_game(game_id);
            let mut game = self.open_games.read(game_id);

            let now = get_block_timestamp();
            if self.settle_timeout(@game, now) {
                return;
            }

            let hit_report = game.defend(defender, status, proof, now);
            self.open_games.entry(game_id).write(game.clone());

            if let Some(report) = hit_report {
                self
                    .emit(
                        AttackResultEvent {
                            game_id,
                            attacker: report.attacker,
                            defender: report.defender,
                            x: report.x,
                            y: report.y,
                            hit: report.hit,
                            destroyed_ship_kind: report.destroyed,
                        },
                    )
            }

            if game.outcome_before_reveal.is_some() {
                self
                    .emit(
                        GameRevealRequestEvent {
                            game_id: game.id, player_a: game.player_a, player_b: game.player_b,
                        },
                    );
            }
        }

        fn reveal(ref self: ContractState, game_id: felt252, ships: Array<Ship>, salt: felt252) {
            let player = self.assert_player_in_game(game_id);
            let mut game = self.open_games.read(game_id);

            let now = get_block_timestamp();
            if self.settle_timeout(@game, now) {
                return;
            }

            let outcome = game.reveal(player, ships, salt, now);

            if let Some(final_outcome) = outcome {
                self.handle_game_over(@game, final_outcome);
            } else {
                self.open_games.entry(game_id).write(game.clone());
            }
        }

        fn claim_timeout(ref self: ContractState, game_id: felt252) {
            self.assert_player_in_game(game_id);
            let game = self.open_games.read(game_id);
            let now = get_block_timestamp();
            assert!(self.settle_timeout(@game, now), "The game is not timed out yet.")
        }

        fn reset(ref self: ContractState) {
            self.ownable.assert_only_owner();

            let next_game_id = self.next_game_id.read();
            if next_game_id == 0 {
                return;
            }

            let mut game_id = next_game_id - 1;
            while game_id != 0 {
                let game = self.open_games.read(game_id);

                self.open_games_per_player.entry(game.player_a).write(0);
                self.open_games_per_player.entry(game.player_b).write(0);

                self.open_games.entry(game_id).write(Default::default());

                game_id -= 1;
            }

            let all_board_sizes = AllBoardSizesTrait::all();
            let zero_address: ContractAddress = 0.try_into().unwrap();
            for board_size in all_board_sizes {
                let size = board_size.size();
                self.open_lobbies.entry(size).write(zero_address);
            }

            self.next_game_id.write(1);

            self.emit(ResetEvent { game_id: 0, timestamp: get_block_timestamp() })
        }

        fn get_next_game_id(self: @ContractState) -> felt252 {
            self.next_game_id.read()
        }

        fn get_timeout_config(self: @ContractState) -> TimeoutConfig {
            Default::default()
        }

        fn get_lobbies(self: @ContractState) -> Lobbies {
            let all_board_sizes = AllBoardSizesTrait::all();
            let mut waitlist: Array<Lobby> = ArrayTrait::new();

            for board_size in all_board_sizes {
                let size = board_size.size();
                let player = self.open_lobbies.entry(size).read();

                if player.is_non_zero() {
                    let lobby = Lobby { player, size: *board_size };
                    waitlist.append(lobby);
                }
            }

            Lobbies { waitlist }
        }
    }

    #[generate_trait]
    impl InternalImpl of InternalTrait {
        fn start_game(
            ref self: ContractState, opponent: ContractAddress, board_size: BoardSize,
        ) -> felt252 {
            let player_a = get_caller_address();
            let player_b = opponent;

            let game_id = self.next_game_id.read();
            let game = GameTrait::new(
                game_id, player_a, player_b, board_size, get_block_timestamp(),
            );

            self.open_games_per_player.entry(player_a).write(game_id);
            self.open_games_per_player.entry(player_b).write(game_id);
            self.open_games.entry(game_id).write(game);
            self.next_game_id.write(game_id + 1);

            self.emit(PlayersAssembledEvent { game_id, player_a, player_b, board_size });

            game_id
        }

        fn assert_player_in_game(self: @ContractState, game_id: felt252) -> ContractAddress {
            let player = get_caller_address();

            let open_game_id = self.open_games_per_player.read(player);
            assert!(
                game_id == open_game_id, "The player {:?} is not playing in {}", player, game_id,
            );

            player
        }

        fn settle_timeout(ref self: ContractState, game: @Game, now: u64) -> bool {
            if let Some(outcome) = game.check_timeout(now) {
                self.handle_game_over(game, outcome);

                return true;
            }

            false
        }

        fn handle_game_over(ref self: ContractState, game: @Game, outcome: Outcome) {
            let game_id = game.id;
            let player_a = game.player_a;
            let player_b = game.player_b;

            self.open_games_per_player.entry(*player_a).write(0);
            self.open_games_per_player.entry(*player_b).write(0);
            self.open_games.entry(*game_id).write(Default::default());

            self
                .emit(
                    GameOverEvent {
                        game_id: *game_id,
                        player_a: *player_a,
                        player_b: *player_b,
                        outcome: outcome,
                    },
                );
        }
    }
}
