
use starkwaves_client::types::error::GameError;
use thiserror::Error as ThisError;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, ThisError)]
pub enum TuiError {
    #[error("{0}")]
    Game(GameError),

    #[error("Unable to send intent, as the channel was closed.")]
    SendIntentError
}

impl <T> From<SendError<T>> for TuiError {
    fn from(_value: SendError<T>) -> Self {
        Self::SendIntentError
    }
}