use starknet_rust::accounts::AccountError;
use starknet_rust::core::types::{ContractExecutionError, StarknetError};
use starknet_rust::providers::ProviderError;
use crate::types::board_size::BoardSize;
use crate::types::{Orientation, ShipKind};
use thiserror::Error as ThisError;

pub use starknet_rust::core::codec::Error as CodecError;
use starknet_rust_tokio_tungstenite::{SubscribeError, SubscriptionReceiveError};

#[derive(Clone, Debug, Eq, ThisError, PartialEq)]
pub enum GameError {
    #[error("A {ship} is not eligible for boards {size} size")]
    ShipIneligible { ship: ShipKind, size: BoardSize },

    #[error("Placing a {ship} in ({x}, {y}) in {orientation} results in out of bounds.")]
    InvalidShipPlacementBounds { ship: ShipKind, x: u8, y: u8, orientation: Orientation },

    #[error("Placing a {ship} in ({x}, {y}) in {orientation} collides in ({xc}, {yc})")]
    InvalidShipPlacementCollides {
        ship: ShipKind,
        x: u8,
        y: u8,
        orientation: Orientation,
        xc: u8,
        yc: u8,
    },

    #[error("You cannot place any more {kind}s for this board size.")]
    InvalidShipPlacementKind { kind: ShipKind },

    #[error("Game is not started yet")]
    GameNotStarted,

    #[error("All ships are not placed in board")]
    BoardNotReady,

    #[error("({x}, {y}) was bombed already")]
    BombedAlready { x: u8, y: u8 },

    #[error("Board is already committed")]
    BoardAlreadyCommitted,

    #[error("All ships are placed in board")]
    AllShipsPlaced,

    #[error("Game is over")]
    GameOver,

    #[error("Cannot create a new game. Player is in another game.")]
    PlayerInGame,

    #[error("You have hit the rate limit.")]
    RateLimited,

    #[error("{error}")]
    ProviderError {
        error: String
    },

    #[error("It is not your turn to attack.")]
    CannotAttack,

    #[error("Invalid input. Expected {expected} but received {received}.")]
    InvalidInput {
        expected: String,
        received: String
    }
}

impl Into<GameError> for ProviderError {
    fn into(self) -> GameError {
        match self {
            ProviderError::StarknetError(inner) => {
                inner.into()
            }
            ProviderError::RateLimited => GameError::RateLimited,
            ProviderError::ArrayLengthMismatch => GameError::ProviderError {
                error: self.to_string()
            },
            ProviderError::Other(internal) => {
                GameError::ProviderError {
                    error: internal.to_string()
                }
            }
        }
    }
}

impl Into<GameError> for StarknetError {
    fn into(self) -> GameError {
        match self {
            StarknetError::ContractError(error_data) => {
                error_data.revert_error.into()
            },
            StarknetError::TransactionExecutionError(error_data) => {
                error_data.execution_error.into()
            },
            _ => GameError::ProviderError { error: self.to_string() },
        }
    }
}

impl Into<GameError> for ContractExecutionError {
    fn into(self) -> GameError {
        match self {
            ContractExecutionError::Nested(nested_error) => {
                nested_error.error.as_ref().clone().into()
            }
            ContractExecutionError::Message(message) => {
                match message.as_str() {
                    s if s.contains("is already in another game.") => GameError::PlayerInGame,
                    _ => GameError::ProviderError { error: message },
                }
            }
        }
    }
}

impl<S> Into<GameError> for AccountError<S>
where
    S: std::fmt::Debug,
{
    fn into(self) -> GameError {
        match self {
            AccountError::Signing(error) => {
                GameError::ProviderError { error: format!("{:?}", error) }
            }
            AccountError::Provider(error) => error.into(),
            AccountError::ClassHashCalculation(error) => {
                GameError::ProviderError { error: error.to_string() }
            }
            AccountError::FeeOutOfRange => {
                GameError::ProviderError { error: "Fee out of range".to_string() }
            }
        }
    }
}

impl Into<GameError> for SubscribeError {
    fn into(self) -> GameError {
        GameError::ProviderError { error: format!("{:?}", self) }
    }
}

impl Into<GameError> for SubscriptionReceiveError {
    fn into(self) -> GameError {
        GameError::ProviderError { error: format!("{:?}", self) }
    }
}

impl Into<GameError> for CodecError {
    fn into(self) -> GameError {
        GameError::ProviderError { error: format!("{:?}", self) }
    }
}

impl Into<GameError> for String {
    fn into(self) -> GameError {
        GameError::ProviderError { error: format!("{:?}", self) }
    }
}