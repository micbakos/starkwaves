use core::fmt::Display;
use starknet::ContractAddress;
use super::ship::ShipKind;

#[derive(Debug, Drop, Serde, Copy)]
pub enum FireStatus {
    Miss: felt252, // Felt is for pedersen(ship_kind.id, salt) 
    Hit: (ShipKind, felt252),
}

impl DisplayImpl of Display<FireStatus> {
    fn fmt(self: @FireStatus, ref f: core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        let display: ByteArray = match self {
            FireStatus::Miss(_) => "Miss",
            FireStatus::Hit(_) => "Hit",
        };
        write!(f, "{display}")
    }
}

#[generate_trait]
pub impl FireStatusImpl of FireStatusTrait {
    fn salted_status(self: @FireStatus) -> felt252 {
        match self {
            FireStatus::Miss(status) => *status,
            FireStatus::Hit((_, status)) => *status,
        }
    }

    fn is_hit(self: @FireStatus) -> bool {
        match self {
            FireStatus::Hit(_) => true,
            FireStatus::Miss(_) => false,
        }
    }
}

#[derive(Debug, Drop, Serde, Copy)]
pub struct HitReport {
    pub attacker: ContractAddress,
    pub defender: ContractAddress,
    pub x: u8,
    pub y: u8,
    pub hit: Option<ShipKind>,
}
