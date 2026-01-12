use crate::types::Ship;
use uuid::Uuid;

/// Represents a cell on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Water,
    Ship(Uuid),
}

impl Cell {
    pub fn ship(&self, ships: &Vec<Ship>) -> Option<Ship> {
        match self {
            Cell::Water => None,
            Cell::Ship(id) => {
                ships.into_iter().find(|ship| ship.id == *id).cloned()
            }
        }
    }
    
    #[cfg(test)]
    pub fn assert_kind(&self, ships: &Vec<Ship>, kind: crate::types::ShipKind) {
        let ship = self.ship(ships).expect("Should be a ship");
        assert_eq!(ship.kind, kind)
    }
}