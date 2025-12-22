use std::collections::HashSet;
use derive_more::Display;

/// Represents different types of ships in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum ShipKind {
    #[display("Super Carrier")]
    SuperCarrier, // Size 6
    #[display("Carrier")]
    Carrier,      // Size 5
    #[display("Battleship")]
    Battleship,   // Size 4
    #[display("Cruiser")]
    Cruiser,      // Size 3
    #[display("Submarine")]
    Submarine,    // Size 3
    #[display("Destoryer")]
    Destroyer,    // Size 2
}

impl ShipKind {
    /// Returns the length of the ship
    pub fn length(&self) -> usize {
        match self {
            ShipKind::SuperCarrier => 6,
            ShipKind::Carrier => 5,
            ShipKind::Battleship => 4,
            ShipKind::Cruiser => 3,
            ShipKind::Submarine => 3,
            ShipKind::Destroyer => 2,
        }
    }

    /// Returns a numeric identifier for the ship (for board representation)
    pub fn id(&self) -> u8 {
        match self {
            ShipKind::Carrier => 1,
            ShipKind::Battleship => 2,
            ShipKind::Cruiser => 3,
            ShipKind::Submarine => 4,
            ShipKind::Destroyer => 5,
            ShipKind::SuperCarrier => 6,
        }
    }
    
    pub fn code(&self) -> &str {
        match self {
            ShipKind::SuperCarrier => "SC",
            ShipKind::Carrier => "CA",
            ShipKind::Battleship => "BA",
            ShipKind::Cruiser => "CR",
            ShipKind::Submarine => "SU",
            ShipKind::Destroyer => "DE",
        }
    }

    pub fn all() -> HashSet<ShipKind> {
        HashSet::from([
            ShipKind::SuperCarrier,
            ShipKind::Carrier,
            ShipKind::Battleship,
            ShipKind::Cruiser,
            ShipKind::Submarine,
            ShipKind::Destroyer,
        ])
    }
}
