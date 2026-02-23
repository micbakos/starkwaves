use cainome::cairo_serde::ContractAddress;
use crate::types::contract::starkwaves::Outcome;

#[derive(Debug, Clone)]
pub enum GameOverOutcome {
    Fair { winner: ContractAddress },
    FailedToProvideProof { cheater: ContractAddress },
    Null
}

impl From<Outcome> for GameOverOutcome {
    fn from(value: Outcome) -> Self {
        match value {
            Outcome::Fair(winner) => Self::Fair { winner },
            Outcome::FailedToProvideProof(cheater) => Self::FailedToProvideProof { cheater },
            Outcome::Null => Self::Null,
        }
    }
}