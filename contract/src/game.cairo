use core::pedersen::pedersen;
use starknet::ContractAddress;
use crate::types::board::{hulls_to_merkle_leaves, ships_to_hulls};
use crate::types::{
    BoardSize, BoardSizeTrait, FireStatus, FireStatusTrait, HitReport, Outcome, OutcomeBeforeReveal,
    OutcomeBeforeRevealTrait, RevealStatus, Ship, ShipKindTrait,
};
use crate::utils::{
    append_bomb_at, contains_bomb_at, get_bomb_offset_at_turn, offset_to_cartesian,
    verify_destructions,
};

#[derive(Debug, Drop, starknet::Store, Clone)]
pub struct Game {
    pub id: felt252,
    pub board_size: BoardSize,
    pub player_a: ContractAddress,
    pub player_b: ContractAddress,
    pub player_a_bombs_on_b: ByteArray,
    pub player_b_bombs_on_a: ByteArray,
    pub player_a_hits_on_b: u8,
    pub player_b_hits_on_a: u8,
    pub player_a_destruction_hash: felt252,
    pub player_b_destruction_hash: felt252,
    pub player_a_root: Option<felt252>,
    pub player_b_root: Option<felt252>,
    pub attacking_player: Option<ContractAddress>,
    pub turn_index: u32,
    pub outcome_before_reveal: Option<OutcomeBeforeReveal>,
    pub player_a_reveal_status: Option<RevealStatus>,
    pub player_b_reveal_status: Option<RevealStatus>,
}

impl GameDefault of Default<Game> {
    fn default() -> Game {
        let zero_address: ContractAddress = 0.try_into().unwrap();
        Game {
            id: 0, // id == 0 means game doesn't exist
            board_size: Default::default(),
            player_a: zero_address,
            player_b: zero_address,
            player_a_bombs_on_b: Default::default(),
            player_b_bombs_on_a: Default::default(),
            player_a_hits_on_b: 0,
            player_b_hits_on_a: 0,
            player_a_destruction_hash: 0,
            player_b_destruction_hash: 0,
            player_a_root: None,
            player_b_root: None,
            attacking_player: None,
            turn_index: 0,
            outcome_before_reveal: None,
            player_a_reveal_status: None,
            player_b_reveal_status: None,
        }
    }
}

#[generate_trait]
pub impl GameImpl of GameTrait {
    fn exists(self: @Game) -> bool {
        *self.id != 0
    }

    fn new(
        id: felt252, player_a: ContractAddress, player_b: ContractAddress, board_size: BoardSize,
    ) -> Game {
        Game {
            id,
            board_size,
            player_a,
            player_b,
            player_a_bombs_on_b: Default::default(),
            player_b_bombs_on_a: Default::default(),
            player_a_hits_on_b: 0,
            player_b_hits_on_a: 0,
            player_a_root: None,
            player_b_root: None,
            player_a_destruction_hash: 0,
            player_b_destruction_hash: 0,
            attacking_player: None,
            turn_index: 0,
            outcome_before_reveal: None,
            player_a_reveal_status: None,
            player_b_reveal_status: None,
        }
    }

    fn commit_root(ref self: Game, player: ContractAddress, root: felt252) {
        if self.player_a == player {
            assert!(
                self.player_a_root.is_none(),
                "Player {:?} has already committed the board.",
                player,
            );
            self.player_a_root = Some(root)
        } else if self.player_b == player {
            assert!(
                self.player_b_root.is_none(),
                "Player {:?} has already committed the board.",
                player,
            );
            self.player_b_root = Some(root)
        } else {
            assert!(false, "User {:?} is not part of the game {}", player, self.id);
        }

        if self.player_a_root.is_some() && self.player_b_root.is_some() {
            self.attacking_player = Some(self.player_a);
        }
    }

    fn register_attack(ref self: Game, player: ContractAddress, x: u8, y: u8) {
        let size = self.board_size.size();
        assert!(x < size && y < size, "Attack on ({}, {}) is out of bounds", x, y);

        if let Some(attacking_player) = self.attacking_player {
            assert!(attacking_player == player, "It is not player's {:?} turn yet.", player);

            let turn = self.turn_index;
            assert!(
                self.bomb_offset_at_turn(@player, turn).is_none(),
                "Player {:?} cannot attack again in this turn.",
                player,
            );

            assert!(!self.is_bombed(@player, x, y), "The ({}, {}) is already bombed", x, y);

            self.append_bomb(player, x, y);
        } else {
            assert!(false, "Player {:?} cannot attack in the game {} yet.", player, self.id);
        }
    }

    fn defend(
        ref self: Game, player: ContractAddress, status: FireStatus, proof: Array<felt252>,
    ) -> Option<HitReport> {
        let attacking_player = self.attacking_player.expect('Attacker not attacked yet.');
        assert!(
            player == self.player_a || player == self.player_b,
            "Player {:?} does not play in game {}",
            player,
            self.id,
        );
        assert!(player != attacking_player, "Attacker cannot defend in this round.");

        let turn = self.turn_index;
        let offset = self
            .bomb_offset_at_turn(@attacking_player, turn)
            .expect('Bomb should have been placed.');
        let defending_root = if attacking_player == self.player_a {
            self.player_b_root
        } else {
            self.player_a_root
        }
            .expect('Commit root should exist.');
        let verified = merkle::verify(status.salted_status(), proof, defending_root, offset);

        if !verified {
            self.outcome_before_reveal = Some(OutcomeBeforeReveal::FailedToProvideProof(player));
            return None;
        }

        let (x, y) = offset_to_cartesian(@self.board_size, offset);
        let mut hit_result = HitReport {
            attacker: attacking_player, defender: player, x, y, hit: false, destroyed: None,
        };
        if let FireStatus::Hit((maybe_destroyed_kind, _)) = status {
            self.increment_success_hits(attacking_player);

            hit_result.hit = true;
            hit_result.destroyed = maybe_destroyed_kind;

            if let Some(destroyed_kind) = maybe_destroyed_kind {
                if player == self.player_a {
                    self
                        .player_a_destruction_hash =
                            pedersen(self.player_a_destruction_hash, destroyed_kind.id().into())
                } else {
                    self
                        .player_b_destruction_hash =
                            pedersen(self.player_b_destruction_hash, destroyed_kind.id().into())
                }
            }

            let won = self.check_won(@attacking_player);
            if won {
                self.outcome_before_reveal = Some(OutcomeBeforeReveal::Fair(attacking_player));
            }
        }

        if self.outcome_before_reveal.is_none() {
            self.attacking_player = Some(player);

            if (player == self.player_a) {
                self.turn_index += 1;
            }
        }

        Some(hit_result)
    }

    fn reveal(
        ref self: Game, player: ContractAddress, ships: Array<Ship>, salt: felt252,
    ) -> Option<Outcome> {
        assert!(self.outcome_before_reveal.is_some(), "The game is not finished yet.");

        let size = self.board_size;
        let ships_span = ships.span();
        let mut hulls = ships_to_hulls(ships_span, @size);

        let board_leaves = hulls_to_merkle_leaves(ref hulls, @size);
        assert!(
            board_leaves.len() == size.leaves(), "The board revealed should be of size {}", size,
        );

        if player == self.player_a {
            assert!(
                self.player_a_reveal_status.is_none(),
                "Player {:?} has already revealed their board.",
                player,
            );
        } else {
            assert!(
                self.player_b_reveal_status.is_none(),
                "Player {:?} has already revealed their board.",
                player,
            );
        }

        let revealed_root = merkle::compute_merkle_root(board_leaves, salt);

        if player == self.player_a {
            let committed = self.player_a_root.expect('Root should have been committed');
            let status = if committed == revealed_root
                && verify_destructions(
                    ref hulls, @size, self.player_a_destruction_hash, @self.player_b_bombs_on_a,
                ) {
                Some(RevealStatus::Real)
            } else {
                Some(RevealStatus::Fake)
            };

            self.player_a_reveal_status = status;
        } else {
            let committed = self.player_b_root.expect('Root should have been committed');
            let status = if committed == revealed_root
                && verify_destructions(
                    ref hulls, @size, self.player_b_destruction_hash, @self.player_a_bombs_on_b,
                ) {
                Some(RevealStatus::Real)
            } else {
                Some(RevealStatus::Fake)
            };

            self.player_b_reveal_status = status;
        }

        if self.player_a_reveal_status.is_none() || self.player_b_reveal_status.is_none() {
            // Still waiting for the other player
            return None;
        }

        Some(self.compute_final_outcome())
    }

    fn defender(self: @Game) -> Option<ContractAddress> {
        self
            .attacking_player
            .map(
                |attacker| {
                    if attacker == *self.player_a {
                        *self.player_b
                    } else {
                        *self.player_a
                    }
                },
            )
    }
}

#[generate_trait]
impl InternalGameImpl of InternalGameTrait {
    fn check_won(self: @Game, attacker: @ContractAddress) -> bool {
        let total_potential_hits = self.board_size.total_hits();
        if attacker == self.player_a {
            total_potential_hits == *self.player_a_hits_on_b
        } else {
            total_potential_hits == *self.player_b_hits_on_a
        }
    }

    fn compute_final_outcome(self: @Game) -> Outcome {
        let outcome_before = (*self.outcome_before_reveal).expect('Outcome before reveal missing');
        let status_a = (*self.player_a_reveal_status).expect('Player A reveal status missing');
        let status_b = (*self.player_b_reveal_status).expect('Player B reveal status missing');

        let a_honest_before = match outcome_before {
            OutcomeBeforeReveal::Fair(_) => true,
            OutcomeBeforeReveal::FailedToProvideProof(cheater) => cheater != *self.player_a,
        };
        let b_honest_before = match outcome_before {
            OutcomeBeforeReveal::Fair(_) => true,
            OutcomeBeforeReveal::FailedToProvideProof(cheater) => cheater != *self.player_b,
        };

        let a_honest_after = status_a == RevealStatus::Real;
        let b_honest_after = status_b == RevealStatus::Real;

        let a_fully_honest = a_honest_before && a_honest_after;
        let b_fully_honest = b_honest_before && b_honest_after;

        if a_fully_honest && b_fully_honest {
            outcome_before.to_outcome()
        } else if a_fully_honest && !b_fully_honest {
            // Player A was honest, Player B cheated at some point
            // Player A wins by opponent's dishonesty
            Outcome::FailedToProvideProof(*self.player_b)
        } else if !a_fully_honest && b_fully_honest {
            // Player B was honest, Player A cheated at some point
            // Player B wins by opponent's dishonesty
            Outcome::FailedToProvideProof(*self.player_a)
        } else {
            // Both players cheated at some point - no winner
            Outcome::Null
        }
    }

    fn increment_success_hits(ref self: Game, attacker: ContractAddress) {
        if attacker == self.player_a {
            self.player_a_hits_on_b += 1;
        } else {
            self.player_b_hits_on_a += 1;
        }
    }
}

#[generate_trait]
pub impl GameBombsImpl of GameBombsTrait {
    fn append_bomb(ref self: Game, attacker: ContractAddress, x: u8, y: u8) {
        if attacker == self.player_a {
            append_bomb_at(ref self.player_a_bombs_on_b, @self.board_size, x, y);
        } else if attacker == self.player_b {
            append_bomb_at(ref self.player_b_bombs_on_a, @self.board_size, x, y);
        }
    }

    fn bomb_offset_at_turn(self: @Game, attacker: @ContractAddress, turn: u32) -> Option<u32> {
        if attacker == self.player_a {
            get_bomb_offset_at_turn(self.player_a_bombs_on_b, self.board_size, turn)
        } else if attacker == self.player_b {
            get_bomb_offset_at_turn(self.player_b_bombs_on_a, self.board_size, turn)
        } else {
            None
        }
    }

    fn is_bombed(self: @Game, attacker: @ContractAddress, x: u8, y: u8) -> bool {
        if attacker == self.player_a {
            contains_bomb_at(self.player_a_bombs_on_b, self.board_size, x, y)
        } else if attacker == self.player_b {
            contains_bomb_at(self.player_b_bombs_on_a, self.board_size, x, y)
        } else {
            false
        }
    }
}
