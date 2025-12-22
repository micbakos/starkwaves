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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ship_lengths() {
        assert_eq!(ShipKind::SuperCarrier.length(), 6);
        assert_eq!(ShipKind::Carrier.length(), 5);
        assert_eq!(ShipKind::Battleship.length(), 4);
        assert_eq!(ShipKind::Cruiser.length(), 3);
        assert_eq!(ShipKind::Submarine.length(), 3);
        assert_eq!(ShipKind::Destroyer.length(), 2);
    }

    #[test]
    fn test_ship_ids() {
        assert_eq!(ShipKind::Carrier.id(), 1);
        assert_eq!(ShipKind::Battleship.id(), 2);
        assert_eq!(ShipKind::Cruiser.id(), 3);
        assert_eq!(ShipKind::Submarine.id(), 4);
        assert_eq!(ShipKind::Destroyer.id(), 5);
        assert_eq!(ShipKind::SuperCarrier.id(), 6);
    }

    #[test]
    fn test_ship_codes() {
        assert_eq!(ShipKind::SuperCarrier.code(), "SC");
        assert_eq!(ShipKind::Carrier.code(), "CA");
        assert_eq!(ShipKind::Battleship.code(), "BA");
        assert_eq!(ShipKind::Cruiser.code(), "CR");
        assert_eq!(ShipKind::Submarine.code(), "SU");
        assert_eq!(ShipKind::Destroyer.code(), "DE");
    }

    #[test]
    fn test_ship_display() {
        assert_eq!(format!("{}", ShipKind::SuperCarrier), "Super Carrier");
        assert_eq!(format!("{}", ShipKind::Carrier), "Carrier");
        assert_eq!(format!("{}", ShipKind::Battleship), "Battleship");
        assert_eq!(format!("{}", ShipKind::Cruiser), "Cruiser");
        assert_eq!(format!("{}", ShipKind::Submarine), "Submarine");
        assert_eq!(format!("{}", ShipKind::Destroyer), "Destoryer"); // Note: typo in original
    }

    #[test]
    fn test_all_ships() {
        let all_ships = ShipKind::all();

        assert_eq!(all_ships.len(), 6);
        assert!(all_ships.contains(&ShipKind::SuperCarrier));
        assert!(all_ships.contains(&ShipKind::Carrier));
        assert!(all_ships.contains(&ShipKind::Battleship));
        assert!(all_ships.contains(&ShipKind::Cruiser));
        assert!(all_ships.contains(&ShipKind::Submarine));
        assert!(all_ships.contains(&ShipKind::Destroyer));
    }

    #[test]
    fn test_ship_ids_are_unique() {
        let all_ships = ShipKind::all();
        let ids: HashSet<u8> = all_ships.iter().map(|ship| ship.id()).collect();

        assert_eq!(ids.len(), 6, "All ship IDs should be unique");
    }

    #[test]
    fn test_ship_codes_are_unique() {
        let all_ships = ShipKind::all();
        let codes: HashSet<&str> = all_ships.iter().map(|ship| ship.code()).collect();

        assert_eq!(codes.len(), 6, "All ship codes should be unique");
    }

    #[test]
    fn test_ship_equality() {
        assert_eq!(ShipKind::Carrier, ShipKind::Carrier);
        assert_ne!(ShipKind::Carrier, ShipKind::Battleship);
    }

    #[test]
    fn test_ships_can_be_hashed() {
        let mut set = HashSet::new();
        set.insert(ShipKind::Carrier);
        set.insert(ShipKind::Carrier); // Duplicate

        assert_eq!(set.len(), 1, "Duplicate ships should not be added to HashSet");
        assert!(set.contains(&ShipKind::Carrier));
    }
}
