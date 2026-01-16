use starknet::{ContractAddress, get_caller_address};
use crate::merkle;
use crate::types::{DefenseReport, FireStatus, FireStatusTrait, Outcome, total_hits};

#[derive(Debug, Drop, starknet::Store, Clone)]
pub struct Game {
    pub id: felt252,
    pub board_size: u8,
    pub player_a: ContractAddress,
    pub player_b: ContractAddress,
    pub player_a_bombs: ByteArray,
    pub player_b_bombs: ByteArray,
    pub player_a_hits: u8,
    pub player_b_hits: u8,
    pub player_a_root: Option<felt252>,
    pub player_b_root: Option<felt252>,
    pub attacking_player: Option<ContractAddress>,
    pub turn_index: u32,
    pub potential_outcome: Option<Outcome>,
}

#[generate_trait]
pub impl GameImpl of GameTrait {
    fn new(
        id: felt252, player_a: ContractAddress, player_b: ContractAddress, board_size: u8,
    ) -> Game {
        assert!(
            board_size == 6 // 6x6
                || board_size == 8 // 8x8
                || board_size == 10 // 10x10
                || board_size == 12 // 12x12
                || board_size == 14 // 14x14
                || board_size == 20, // 20x20
            "Board is not a valid size.",
        );

        Game {
            id,
            board_size,
            player_a,
            player_b,
            player_a_bombs: Default::default(),
            player_b_bombs: Default::default(),
            player_a_hits: 0,
            player_b_hits: 0,
            player_a_root: None,
            player_b_root: None,
            attacking_player: None,
            turn_index: 0,
            potential_outcome: None,
        }
    }

    fn commit_root(ref self: Game, player: ContractAddress, root: felt252) {
        let is_player_a = self.player_a == player;

        if is_player_a {
            assert!(
                self.player_a_root.is_none(),
                "Player {:?} has already committed the board.",
                player,
            );
            self.player_a_root = Some(root)
        } else {
            assert!(
                self.player_b_root.is_none(),
                "Player {:?} has already committed the board.",
                player,
            );
            self.player_b_root = Some(root)
        }

        if self.player_a_root.is_some() && self.player_b_root.is_some() {
            self.attacking_player = Some(self.player_a);
        }
    }

    fn register_attack(ref self: Game, x: u8, y: u8) {
        assert!(
            x < self.board_size && y < self.board_size, "Attack on ({}, {}) is out of bounds", x, y,
        );
        let player_address = get_caller_address();

        if let Some(attacking_player) = self.attacking_player {
            assert!(
                attacking_player == player_address,
                "It is not player's {:?} turn yet.",
                player_address,
            );

            assert!(
                self.bomb_offset_in_current_turn(@player_address).is_none(),
                "Player {:?} cannot attack again in this turn.",
                player_address,
            );

            assert!(!self.is_bombed(@player_address, x, y), "The ({}, {}) is already bombed", x, y);

            self.append_bomb(player_address, x, y);
        } else {
            assert!(false, "Players on game {} have not yet committed their boards.", self.id);
        }
    }

    /// Handles the response of the defender.
    /// Returns true when the game is over and both players are required to reveal their boards.
    fn defend(ref self: Game, status: FireStatus, proof: Array<felt252>) -> DefenseReport {
        let defender_address = get_caller_address();

        let attacking_player = self.attacking_player.expect('Attacker not attacked yet.');
        assert!(
            defender_address == self.player_a || defender_address == self.player_b,
            "Player {:?} does not play in game {}",
            defender_address,
            self.id,
        );
        assert!(defender_address != attacking_player, "Attacker cannot defend in this round.");

        let offset = self
            .bomb_offset_in_current_turn(@attacking_player)
            .expect('Bomb should have been placed.');
        let defending_root = if attacking_player == self.player_a {
            self.player_b_root
        } else {
            self.player_a_root
        }
            .expect('Commit root should exist.');
        let (x, y) = self.offset_to_cartesian(offset);

        let verified = merkle::verify(
            status.salted_status(), proof, defending_root, offset.try_into().unwrap(),
        );

        if !verified {
            self.potential_outcome = Some(Outcome::FailedToProvideProof(defender_address));
            return DefenseReport {
                reveal_boards: true, attacker: attacking_player, defender: defender_address, x, y,
            };
        }

        if status.is_hit() {
            self.increment_success_hits(attacking_player);
        }

        let won = self.check_won(@attacking_player);
        if won {
            self.potential_outcome = Some(Outcome::Fair(attacking_player));
        } else {
            self.attacking_player = Some(defender_address);

            if (defender_address == self.player_a) {
                self.turn_index += 1;
            }
        }

        DefenseReport {
            reveal_boards: won, attacker: attacking_player, defender: defender_address, x, y,
        }
    }

    fn increment_success_hits(ref self: Game, attacker: ContractAddress) {
        if attacker == self.player_a {
            self.player_a_hits += 1;
        } else {
            self.player_b_hits += 1;
        }
    }

    fn check_won(self: @Game, attacker: @ContractAddress) -> bool {
        let total_potential_hits = total_hits(*self.board_size);
        if attacker == self.player_a {
            total_potential_hits == *self.player_a_hits
        } else {
            total_potential_hits == *self.player_b_hits
        }
    }
}

#[generate_trait]
pub impl GameBombsImpl of GameBombsTrait {
    fn offset_bytes(self: @Game, x: u8, y: u8) -> (u8, u8) {
        let board_size = *self.board_size;

        let rows_offset: u32 = x.into() * board_size.into();
        let offset = rows_offset + y.into();

        let high_byte: u8 = (offset / 256).try_into().unwrap();
        let low_byte: u8 = (offset % 256).try_into().unwrap();

        (high_byte, low_byte)
    }

    fn append_bomb(ref self: Game, attacker: ContractAddress, x: u8, y: u8) {
        let (high, low) = self.offset_bytes(x, y);

        if attacker == self.player_a {
            self.player_a_bombs.append_byte(high);
            self.player_a_bombs.append_byte(low);
        } else if attacker == self.player_b {
            self.player_b_bombs.append_byte(high);
            self.player_b_bombs.append_byte(low);
        }
    }

    fn bomb_offset_in_current_turn(self: @Game, attacker: @ContractAddress) -> Option<u32> {
        let turn = *self.turn_index;

        if attacker == self.player_a {
            Some(self.player_a_bombs)
        } else if attacker == self.player_b {
            Some(self.player_b_bombs)
        } else {
            None
        }
            .and_then(
                |b| {
                    let mut offset: u32 = 0;
                    if let Some(high) = b.at(turn * 2) {
                        offset = high.into() * 256;
                    } else {
                        return None;
                    }

                    if let Some(low) = b.at((turn * 2) + 1) {
                        offset += low.into();
                    } else {
                        return None;
                    }

                    Some(offset)
                },
            )
    }

    fn offset_to_cartesian(self: @Game, offset: u32) -> (u8, u8) {
        let board_size: u32 = (*self.board_size).into();
        let x: u8 = (offset / board_size).try_into().unwrap();
        let y: u8 = (offset % board_size).try_into().unwrap();
        (x, y)
    }

    fn is_bombed(self: @Game, attacker: @ContractAddress, x: u8, y: u8) -> bool {
        let (high, low) = self.offset_bytes(x, y);

        if attacker == self.player_a {
            self.player_a_bombs.contains_bomb(high, low)
        } else if attacker == self.player_b {
            self.player_b_bombs.contains_bomb(high, low)
        } else {
            false
        }
    }

    fn contains_bomb(self: @ByteArray, high: u8, low: u8) -> bool {
        let len = self.len();
        let mut i = 0;

        while i < len {
            if i + 1 < len {
                let _high = self.at(i).unwrap();
                let _low = self.at(i + 1).unwrap();
                if high == _high && low == _low {
                    return true;
                }
            }
            i = i + 2;
        }

        false
    }
}
