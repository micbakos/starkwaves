use crate::types::contract::starkwaves::Outcome;
use cainome::cairo_serde::ContractAddress;

#[derive(Debug, Clone)]
pub enum GameOverOutcome {
    Won,
    Lost(LossReason),
}

#[derive(Debug, Clone)]
pub enum LossReason {
    FairGame,
    FailedToProvideProof,
}

impl GameOverOutcome {
    pub fn from(outcome: Outcome, player_address: ContractAddress) -> Self {
        match outcome {
            Outcome::Fair(winner) => {
                if winner == player_address {
                    GameOverOutcome::Won
                } else {
                    GameOverOutcome::Lost(LossReason::FairGame)
                }
            }
            Outcome::FailedToProvideProof(_) => {
                GameOverOutcome::Lost(LossReason::FailedToProvideProof)
            }
            Outcome::Null => {
                GameOverOutcome::Lost(LossReason::FailedToProvideProof)
            }
        }
    }
}