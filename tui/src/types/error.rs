use derive_more::From;
use starkwaves_client::types::error::GameError;
use thiserror::Error as ThisError;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, ThisError, From)]
pub enum TuiError {
    #[error("{0}")]
    Game(GameError),

    #[error("Unable to send intent, as the channel was closed.")]
    SendIntentError,

    #[error("Unable to read from storage. {0}")]
    #[from(skip)]
    FailedToReadFromStorage(String),

    #[error("Unable to write to storage. {0}")]
    #[from(skip)]
    FailedToWriteToStorage(String),

    #[cfg(debug_assertions)]
    #[error("Failed to read env")]
    FailedToReadAccountKeysFromEnv
}

impl <T> From<SendError<T>> for TuiError {
    fn from(_value: SendError<T>) -> Self {
        Self::SendIntentError
    }
}
