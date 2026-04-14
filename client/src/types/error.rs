use crate::types::board_size::BoardSize;
use crate::types::{Orientation, ShipKind};
use starknet_rust::accounts::AccountError;
use starknet_rust::core::types::{ContractExecutionError, StarknetError};
use starknet_rust::providers::ProviderError;
use thiserror::Error as ThisError;

pub use starknet_rust::core::codec::Error as CodecError;
use starknet_rust_tokio_tungstenite::{SubscribeError, SubscriptionReceiveError};

#[derive(Debug, ThisError)]
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

    #[error("All ships are already placed in board")]
    AllShipsAlreadyPlaced,

    #[error("Game is over")]
    GameOver,

    #[error("Cannot create a new game. Player is in another game.")]
    PlayerInGame,

    #[error("Transaction {tx_hash} was reverted due to: {reason}")]
    TxReverted {
        tx_hash: String,
        reason: String
    },

    #[error("{0}")]
    InvalidState(String),

    #[error("{0}")]
    StarknetProviderError(ProviderError),

    #[error("Subscription error: {0}")]
    StarknetSubscriptionError(String),

    #[error("Account error: {0}")]
    AccountError(String),

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
            ProviderError::StarknetError(error) => error.into(),
            _ => GameError::StarknetProviderError(self)
        }
    }
}

impl Into<GameError> for StarknetError {
    fn into(self) -> GameError {
        match self.clone() {
            StarknetError::ContractError(contract_error) => {
                match contract_error.revert_error {
                    ContractExecutionError::Nested(_) => {
                        GameError::StarknetProviderError(ProviderError::StarknetError(self))
                    }
                    ContractExecutionError::Message(message) => {
                        match message.as_str() {
                            s if s.contains("is already in another game.") => GameError::PlayerInGame,
                            _ => GameError::StarknetProviderError(ProviderError::StarknetError(self))
                        }
                    }
                }
            }
            _ => GameError::StarknetProviderError(ProviderError::StarknetError(self))
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
                GameError::AccountError(format!("{:?}", error))
            }
            AccountError::Provider(error) => error.into(),
            _ => GameError::AccountError(format!("{:?}", self))
        }
    }
}

impl Into<GameError> for SubscribeError {
    fn into(self) -> GameError {
        GameError::StarknetSubscriptionError(format!("{:?}", self))
    }
}

impl Into<GameError> for SubscriptionReceiveError {
    fn into(self) -> GameError {
        GameError::StarknetSubscriptionError(format!("{:?}", self))
    }
}