use starknet::ContractAddress;
use super::BoardSize;

#[derive(Debug, Drop, Serde)]
pub struct Lobbies {
    pub waitlist: Array<Lobby>,
}

#[derive(Debug, Drop, Serde)]
pub struct Lobby {
    pub player: ContractAddress,
    pub size: BoardSize,
}
