use crate::types::contract::starkwaves::Outcome;
use cainome::cairo_serde::ContractAddress;

#[derive(Debug, Clone)]
pub enum GameOverOutcome {
    Won(Reason),
    Lost(Reason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    FairGame,
    FailedToProvideProof,
}

impl GameOverOutcome {
    pub fn from(outcome: Outcome, player_address: ContractAddress) -> Self {
        match outcome {
            Outcome::Fair(winner) => {
                if winner == player_address {
                    GameOverOutcome::Won(Reason::FairGame)
                } else {
                    GameOverOutcome::Lost(Reason::FairGame)
                }
            }
            Outcome::FailedToProvideProof(cheater) => {
                if cheater == player_address {
                    GameOverOutcome::Lost(Reason::FailedToProvideProof)
                } else {
                    GameOverOutcome::Won(Reason::FailedToProvideProof)
                }
            }
            Outcome::Null => {
                GameOverOutcome::Lost(Reason::FailedToProvideProof)
            }
        }
    }
}