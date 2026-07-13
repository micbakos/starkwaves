use crate::types::Ship;
use crate::types::board_size::BoardSize;
use crate::types::contract::generated;
use crate::types::fire_report::{FireReport, FireStatus};
use cainome::cairo_serde::CairoSerde;
use starknet_rust::macros::selector;
use starknet_rust_core::types::{Call, Felt, FunctionCall};

pub struct Starkwaves {
    address: Felt,
}

impl Starkwaves {
    pub fn new(address: Felt) -> Self {
        Self { address }
    }

    pub fn get_lobbies(&self) -> FunctionCall {
        FunctionCall {
            contract_address: self.address,
            entry_point_selector: selector!("get_lobbies"),
            calldata: vec![],
        }
    }

    pub fn request_start_game(&self, board_size: &BoardSize) -> Call {
        let board_size: generated::BoardSize = (*board_size).into();
        Call {
            to: self.address,
            selector: selector!("request_start_game"),
            calldata: generated::BoardSize::cairo_serialize(&board_size),
        }
    }

    pub fn attack(&self, game_id: &Felt, x: &u8, y: &u8) -> Call {
        let mut calldata = vec![];
        calldata.extend(Felt::cairo_serialize(game_id));
        calldata.extend(u8::cairo_serialize(x));
        calldata.extend(u8::cairo_serialize(y));
        Call {
            to: self.address,
            selector: selector!("attack"),
            calldata,
        }
    }

    pub fn commit_board(&self, root: &Felt, game_id: &Felt) -> Call {
        let mut calldata = vec![];
        calldata.extend(Felt::cairo_serialize(root));
        calldata.extend(Felt::cairo_serialize(game_id));
        Call {
            to: self.address,
            selector: selector!("commit_board"),
            calldata,
        }
    }

    pub fn defend(
        &self,
        game_id: &Felt,
        report: &FireReport,
        salt: u64,
        proof: &Vec<Felt>,
    ) -> Call {
        let status: generated::FireStatus = Self::fire_status(report, salt);
        let mut calldata = vec![];
        calldata.extend(Felt::cairo_serialize(game_id));
        calldata.extend(generated::FireStatus::cairo_serialize(&status));
        calldata.extend(Vec::<Felt>::cairo_serialize(proof));
        Call {
            to: self.address,
            selector: selector!("defend"),
            calldata,
        }
    }

    pub fn reveal(&self, game_id: &Felt, ships: &Vec<Ship>, salt: &Felt) -> Call {
        let ships: Vec<generated::Ship> = ships.iter().map(|ship| (*ship).into()).collect();

        let mut calldata = vec![];
        calldata.extend(Felt::cairo_serialize(game_id));
        calldata.extend(Vec::<generated::Ship>::cairo_serialize(&ships));
        calldata.extend(Felt::cairo_serialize(salt));
        Call {
            to: self.address,
            selector: selector!("reveal"),
            calldata,
        }
    }

    pub fn claim_timeout(&self, game_id: &Felt) -> Call {
        Call {
            to: self.address,
            selector: selector!("claim_timeout"),
            calldata: Felt::cairo_serialize(game_id),
        }
    }

    fn fire_status(report: &FireReport, salt: u64) -> generated::FireStatus {
        match &report.status {
            FireStatus::Miss => {
                let status = report.status.salted(salt);
                generated::FireStatus::Miss(status)
            }
            FireStatus::Hit(_) => {
                let status = report.status.salted(salt);
                generated::FireStatus::Hit((report.ship_destroyed.map(|k| k.kind.into()), status))
            }
        }
    }
}
