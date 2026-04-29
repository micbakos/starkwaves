use crate::types::contract::starkwaves::{
    Event, Orientation as ContractOrientation, Ship as ContractShip, ShipKind as ContractShipKind,
};
use crate::types::error::GameError;
use crate::types::{Orientation, Ship, ShipKind};
use starknet_rust::core::types::TransactionReceipt;
use starknet_rust::core::types::{Event as StarknetEvent, Felt};
use starknet_rust::macros::selector;

pub trait IntoEvents {
    fn into_events(self) -> Result<Vec<Event>, GameError>;
}

impl IntoEvents for TransactionReceipt {
    fn into_events(self) -> Result<Vec<Event>, GameError> {
        let events: Vec<StarknetEvent> = if let TransactionReceipt::Invoke(invoke) = self {
            invoke.events
        } else {
            return Err(GameError::InvalidState(format!(
                "Expected TransactionReceipt but instead received {:?}",
                self
            )));
        };

        Ok(events
            .iter()
            .filter_map(|event| Event::try_from(event).ok())
            .collect::<Vec<_>>())
    }
}

impl Into<ContractShip> for Ship {
    fn into(self) -> ContractShip {
        ContractShip {
            kind: self.kind.into(),
            x: self.x,
            y: self.y,
            orientation: self.orientation.into(),
        }
    }
}

impl Into<ContractOrientation> for Orientation {
    fn into(self) -> ContractOrientation {
        match self {
            Orientation::Horizontal => ContractOrientation::Horizontal,
            Orientation::Vertical => ContractOrientation::Vertical,
        }
    }
}

impl Into<ContractShipKind> for ShipKind {
    fn into(self) -> ContractShipKind {
        match self {
            ShipKind::Carrier => ContractShipKind::Carrier,
            ShipKind::Battleship => ContractShipKind::Battleship,
            ShipKind::Cruiser => ContractShipKind::Cruiser,
            ShipKind::Submarine => ContractShipKind::Submarine,
            ShipKind::Destroyer => ContractShipKind::Destroyer,
            ShipKind::SuperCarrier => ContractShipKind::SuperCarrier,
        }
    }
}

impl Into<ShipKind> for ContractShipKind {
    fn into(self) -> ShipKind {
        match self {
            ContractShipKind::Carrier => ShipKind::Carrier,
            ContractShipKind::Battleship => ShipKind::Battleship,
            ContractShipKind::Cruiser => ShipKind::Cruiser,
            ContractShipKind::Submarine => ShipKind::Submarine,
            ContractShipKind::Destroyer => ShipKind::Destroyer,
            ContractShipKind::SuperCarrier => ShipKind::SuperCarrier,
        }
    }
}

pub fn in_lobby_event_keys() -> Vec<Vec<Felt>> {
    vec![
        vec![
            selector!("PlayersAssembled"),
            selector!("Reset"),
        ],
    ]
}

pub fn in_game_event_keys(game_id: Felt) -> Vec<Vec<Felt>> {
    vec![
        vec![
            selector!("GameStarted"),
            selector!("Attack"),
            selector!("AttackResult"),
            selector!("GameRevealRequest"),
            selector!("GameOver"),
            selector!("Reset"),
        ],
        vec![game_id, Felt::ZERO], // Felt::ZERO is for Reset
    ]
}
