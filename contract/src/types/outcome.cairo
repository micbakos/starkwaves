use starknet::ContractAddress;

#[derive(Debug, Drop, Serde, Copy, starknet::Store)]
pub enum OutcomeBeforeReveal {
    #[default]
    Fair: ContractAddress,
    FailedToProvideProof: ContractAddress,
}

#[generate_trait]
pub impl OutcomeBeforeRevealImpl of OutcomeBeforeRevealTrait {
    fn to_outcome(self: OutcomeBeforeReveal) -> Outcome {
        match self {
            OutcomeBeforeReveal::Fair(winner) => Outcome::Fair(winner),
            OutcomeBeforeReveal::FailedToProvideProof(cheater) => Outcome::FailedToProvideProof(
                cheater,
            ),
        }
    }
}

#[derive(Debug, Drop, Serde, Copy, PartialEq, starknet::Store)]
pub enum RevealStatus {
    #[default]
    Real,
    Fake,
}

#[derive(Debug, Drop, Serde, Copy)]
pub enum Outcome {
    #[default]
    Fair: ContractAddress,
    FailedToProvideProof: ContractAddress,
    Null,
}
