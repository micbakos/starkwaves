use starknet::ContractAddress;
use crate::types::{BoardSize, Outcome, ShipKind};

#[derive(Drop, starknet::Event, Serde)]
pub struct PlayerEnteredLobbyEvent {
    #[key]
    pub lobby: BoardSize,
    pub player: ContractAddress,
}

#[derive(Drop, starknet::Event, Serde)]
pub struct PlayersAssembledEvent {
    #[key]
    pub game_id: felt252,
    pub player_a: ContractAddress,
    pub player_b: ContractAddress,
    pub board_size: BoardSize,
}

#[derive(Drop, starknet::Event, Serde)]
pub struct GameStartedEvent {
    #[key]
    pub game_id: felt252,
    pub attacker: ContractAddress,
    pub defender: ContractAddress,
}

#[derive(Drop, starknet::Event, Serde)]
pub struct AttackEvent {
    #[key]
    pub game_id: felt252,
    pub player: ContractAddress,
    pub x: u8,
    pub y: u8,
}

#[derive(Drop, starknet::Event, Serde)]
pub struct AttackResultEvent {
    #[key]
    pub game_id: felt252,
    pub attacker: ContractAddress,
    pub defender: ContractAddress,
    pub x: u8,
    pub y: u8,
    pub hit: bool,
    pub destroyed_ship_kind: Option<ShipKind>,
}

#[derive(Drop, starknet::Event, Serde)]
pub struct GameRevealRequestEvent {
    #[key]
    pub game_id: felt252,
    pub player_a: ContractAddress,
    pub player_b: ContractAddress,
}


#[derive(Drop, starknet::Event, Serde)]
pub struct GameOverEvent {
    #[key]
    pub game_id: felt252,
    pub player_a: ContractAddress,
    pub player_b: ContractAddress,
    pub outcome: Outcome,
}

#[derive(Drop, starknet::Event, Serde)]
pub struct ResetEvent {
    #[key]
    pub game_id: felt252, // always 0
    pub timestamp: u64,
}
