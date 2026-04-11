use crate::types::contract::starkwaves::Event;
use crate::types::error::GameError;
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
            return Err(GameError::ProviderError {
                error: format!(
                    "Expected TransactionReceipt but instead received {:?}",
                    self
                ),
            });
        };

        Ok(events
            .iter()
            .filter_map(|event| Event::try_from(event).ok())
            .collect::<Vec<_>>())
    }
}

pub fn in_lobby_event_keys() -> Vec<Vec<Felt>> {
    vec![
        vec![
            selector!("PlayersAssembled")
        ]
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
        ],
        vec![game_id],
    ]
}