use crate::Ship;
use crate::types::{BoardSize, FireStatus};

#[starknet::interface]
pub trait IStarkwaves<TContractState> {
    fn request_start_game(ref self: TContractState, board_size: BoardSize) -> Option<felt252>;

    fn commit_board(ref self: TContractState, root: felt252, game_id: felt252);

    fn attack(ref self: TContractState, game_id: felt252, x: u8, y: u8);

    fn defend(
        ref self: TContractState, game_id: felt252, status: FireStatus, proof: Array<felt252>,
    );

    fn reveal(ref self: TContractState, game_id: felt252, ships: Array<Ship>, salt: felt252);

    fn reset(ref self: TContractState);

    fn get_next_game_id(self: @TContractState) -> felt252;
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
    use starknet::{ContractAddress, get_caller_address};
    use crate::events::{
        AttackEvent, AttackResultEvent, GameOverEvent, GameRevealRequestEvent, GameStartedEvent,
        PlayerEnteredLobbyEvent, PlayersAssembledEvent,
    };
    use crate::game::{Game, GameTrait};
    use crate::types::{AllBoardSizesTrait, BoardSizeTrait};
    use super::{*, BoardSize, FireStatus};

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
            game.commit_root(player, root);
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

            game.register_attack(player, x, y);

            self.open_games.entry(game_id).write(game);
            self.emit(AttackEvent { game_id, player, x, y })
        }

        fn defend(
            ref self: ContractState, game_id: felt252, status: FireStatus, proof: Array<felt252>,
        ) {
            let defender = self.assert_player_in_game(game_id);
            let mut game = self.open_games.read(game_id);

            let hit_report = game.defend(defender, status, proof);
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
                            ship_kind: report.hit,
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

            let outcome = game.reveal(player, ships, salt);

            if let Some(final_outcome) = outcome {
                self.open_games_per_player.entry(game.player_a).write(0);
                self.open_games_per_player.entry(game.player_b).write(0);
                self.open_games.entry(game_id).write(Default::default());

                self
                    .emit(
                        GameOverEvent {
                            game_id: game.id,
                            player_a: game.player_a,
                            player_b: game.player_b,
                            outcome: final_outcome,
                        },
                    );
            } else {
                self.open_games.entry(game_id).write(game.clone());
            }
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
        }

        fn get_next_game_id(self: @ContractState) -> felt252 {
            self.next_game_id.read()
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
            let game = GameTrait::new(game_id, player_a, player_b, board_size);

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
    }
}
