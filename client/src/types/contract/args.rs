use crate::types::board_size::BoardSize;
use crate::types::error::CodecError;
use crate::types::fire_report::{FireReport, FireStatus};
use starknet::core::codec::Encode;
use starknet::core::crypto::pedersen_hash;
use starknet::core::types::Felt;

#[derive(Encode, Debug)]
pub struct StartGameArgs {
    opponent: Felt,
    board_size: u8
}

impl StartGameArgs {
    pub fn new(opponent: Felt, board_size: BoardSize) -> Self {
        Self {
            opponent,
            board_size: board_size.size().into(),
        }
    }
}

impl TryInto<Vec<Felt>> for StartGameArgs {
    type Error = CodecError;

    fn try_into(self) -> Result<Vec<Felt>, Self::Error> {
        let mut serialized = vec![];
        self.encode(&mut serialized).map(|()| serialized)
    }
}

#[derive(Encode, Debug)]
pub struct CommitBoardArgs {
    pub root: Felt,
    pub game_id: Felt
}

impl TryInto<Vec<Felt>> for CommitBoardArgs {
    type Error = CodecError;

    fn try_into(self) -> Result<Vec<Felt>, Self::Error> {
        let mut serialized = vec![];
        self.encode(&mut serialized).map(|()| serialized)
    }
}

#[derive(Encode, Debug)]
pub struct AttackArgs {
    pub game_id: Felt,
    pub x: u8,
    pub y: u8,
}

impl TryInto<Vec<Felt>> for AttackArgs {
    type Error = CodecError;

    fn try_into(self) -> Result<Vec<Felt>, Self::Error> {
        let mut serialized = vec![];
        self.encode(&mut serialized).map(|()| serialized)
    }
}

#[derive(Encode, Debug)]
pub struct DefendArgs {
    game_id: Felt,
    status: FireStatusArg,
    proof: Vec<Felt>,
}

impl DefendArgs {
    pub fn new(
        game_id: Felt,
        report: &FireReport,
        salt: u64,
    ) -> Self {
        Self {
            game_id,
            status: FireStatusArg::from(&report.status, salt),
            proof: report.proof.clone(),
        }
    }
}

impl TryInto<Vec<Felt>> for DefendArgs {
    type Error = CodecError;

    fn try_into(self) -> Result<Vec<Felt>, Self::Error> {
        let mut serialized = vec![];
        self.encode(&mut serialized).map(|()| serialized)
    }
}

#[derive(Encode, Debug)]
pub enum FireStatusArg {
    Miss(Felt),
    Hit(u8, Felt)
}

impl FireStatusArg {
    pub fn from(status: &FireStatus, salt: u64) -> Self {
        let salt = Felt::from(salt);
        match status {
            FireStatus::Miss => {
                let status = pedersen_hash(&Felt::ZERO, &salt);
                Self::Miss(status)
            }
            FireStatus::Hit(kind) => {
                let id = Felt::from(kind.id());
                let status = pedersen_hash(&id, &salt);
                Self::Hit(kind.clone().id(), status)
            }
        }
    }
}

#[derive(Encode, Debug)]
pub struct RevealArgs {
    pub game_id: Felt,
    pub board: Vec<u8>,
    pub salt: Felt
}

impl TryInto<Vec<Felt>> for RevealArgs {
    type Error = CodecError;

    fn try_into(self) -> Result<Vec<Felt>, Self::Error> {
        let mut serialized = vec![];
        self.encode(&mut serialized).map(|()| serialized)
    }
}




