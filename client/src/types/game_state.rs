use enum_as_inner::EnumAsInner;
use starknet::core::types::Felt;

#[derive(Debug, EnumAsInner)]
pub enum GameState {
    PlacingShips,
    Playing {
        attacking_player: Felt,
        current_attack: Option<(u8, u8)>
    },
    Ended
}