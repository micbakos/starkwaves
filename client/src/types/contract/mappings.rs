use crate::types::contract::starkwaves::Event;
use crate::types::error::GameError;
use starknet::core::types::TransactionReceipt;
use starknet::core::types::{Event as StarknetEvent, Felt};
use starknet::macros::selector;

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
            game_id
        ],
        vec![
            selector!("Attack"),
            game_id
        ],
        vec![
            selector!("Hit"),
            game_id
        ],
        vec![
            selector!("GameRevealRequest"),
            game_id
        ],
        vec![
            selector!("GameOver"),
            game_id
        ],
    ]
}