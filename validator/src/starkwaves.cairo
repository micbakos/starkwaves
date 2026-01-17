use starknet::ContractAddress;
use crate::types::FireStatus;

#[starknet::interface]
pub trait IStarkwaves<TContractState> {
    fn start_game(ref self: TContractState, opponent: ContractAddress, board_size: u8) -> felt252;

    fn commit_board(ref self: TContractState, root: felt252, game_id: felt252);

    fn attack(ref self: TContractState, game_id: felt252, x: u8, y: u8);

    fn defend(
        ref self: TContractState, game_id: felt252, status: FireStatus, proof: Array<felt252>,
    );
    // opponent responds hit or miss with proof => game verifies and updates num of hits per player
//         * the first one who reaches max hits should win
//         * game asks both players to reveal their boards.

    // player A => reveals board => game checks legitimacy, if not player automatically loses
// possible win player B => reveals board => game checks legitimacy, if not player automatically
// loses possible win

    // when both players are verified then the game decides the winner. Game ends

}
// Game
// id
// board_size
// status: DOCKING, STARTED, ENDED
// player_a
// player_b
// player_a_board_root
// player_b_board_root
// player_a_bombs
// player_b_bombs

#[starknet::contract]
pub mod Starkwaves {
    use starknet::event::EventEmitter;
    use starknet::storage::{
        Map, StorageMapReadAccess, StoragePathEntry, StoragePointerReadAccess,
        StoragePointerWriteAccess,
    };
    use starknet::{ContractAddress, get_caller_address};
    use crate::events::{
        AttackEvent, GameOverEvent, GameRevealRequestEvent, GameStartedEvent, HitEvent,
        PlayersAssembledEvent,
    };
    use crate::game::{Game, GameTrait};
    use super::{*, FireStatus};

    #[storage]
    struct Storage {
        next_game_id: felt252,
        open_games: Map<felt252, Game>,
        open_games_per_player: Map<ContractAddress, felt252>,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        PlayersAssembled: PlayersAssembledEvent,
        GameStarted: GameStartedEvent,
        Attack: AttackEvent,
        Hit: HitEvent,
        GameRevealRequest: GameRevealRequestEvent,
        GameOver: GameOverEvent,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        self.next_game_id.write(1);
    }

    #[abi(embed_v0)]
    impl StarkwavesImpl of super::IStarkwaves<ContractState> {
        fn start_game(
            ref self: ContractState, opponent: ContractAddress, board_size: u8,
        ) -> felt252 {
            let player_a = get_caller_address();
            let player_b = opponent;

            let game_id = self.open_games_per_player.entry(player_a).read();
            assert!(game_id == 0, "Player {:?} is already in another game.", player_a);
            let game_id = self.open_games_per_player.entry(player_b).read();
            assert!(game_id == 0, "Player {:?} is already in another game.", player_b);

            let game_id = self.next_game_id.read();
            let game = GameTrait::new(game_id, player_a, player_b, board_size);

            self.open_games_per_player.entry(player_a).write(game_id);
            self.open_games_per_player.entry(player_b).write(game_id);
            self.open_games.entry(game_id).write(game);
            self.next_game_id.write(game_id + 1);

            self.emit(PlayersAssembledEvent { game_id, player_a, player_b });

            game_id
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

            let hit_result = game.defend(defender, status, proof);
            self.open_games.entry(game_id).write(game.clone());

            if let Some(hit) = hit_result {
                self
                    .emit(
                        HitEvent {
                            game_id,
                            attacker: hit.attacker,
                            defender: hit.defender,
                            x: hit.x,
                            y: hit.y,
                            ship_kind: hit.ship_kind,
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
    }

    #[generate_trait]
    impl InternalImpl of InternalTrait {
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
