use starknet::ContractAddress;
use crate::types::ShipKind;

#[derive(Drop, starknet::Event, Serde)]
pub struct PlayersAssembledEvent {
    #[key]
    pub game_id: felt252,
    pub player_a: ContractAddress,
    pub player_b: ContractAddress,
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
pub struct HitEvent {
    #[key]
    pub game_id: felt252,
    pub attacker: ContractAddress,
    pub defender: ContractAddress,
    pub x: u8,
    pub y: u8,
    pub ship_kind: ShipKind,
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
}
