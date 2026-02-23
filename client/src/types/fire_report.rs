use crate::types::{Ship, ShipKind};
use std::fmt::{Display, Formatter};
use starknet::core::crypto::pedersen_hash;
use starknet::core::types::Felt;
use crate::types::contract::starkwaves::FireStatus as SaltedFireStatus;

#[derive(Debug)]
pub struct FireReport {
    pub status: FireStatus,
    pub ship_destroyed: Option<Ship>,
    pub proof: Vec<Felt>
}

#[derive(Debug, PartialEq, Eq)]
pub enum FireStatus {
    Miss,
    Hit(ShipKind)
}

impl FireReport {
    pub fn miss(proof: Vec<Felt>) -> FireReport {
        FireReport {
            status: FireStatus::Miss,
            ship_destroyed: None,
            proof
        }
    }

    pub fn hit(kind: ShipKind, proof: Vec<Felt>) -> FireReport {
        FireReport {
            status: FireStatus::Hit(kind),
            ship_destroyed: None,
            proof
        }
    }

    pub fn hit_with_destruction(ship: Ship, proof: Vec<Felt>) -> FireReport {
        FireReport {
            status: FireStatus::Hit(ship.kind),
            ship_destroyed: Some(ship),
            proof
        }
    }

    pub fn salted_status_value(&self, salt: u64) -> Felt {
        let status = match &self.status {
            FireStatus::Miss => 0,
            FireStatus::Hit(kind) => kind.id()
        };
        pedersen_hash(&Felt::from(status), &Felt::from(salt))
    }

    pub fn salted_fire_status(&self, salt: u64) -> SaltedFireStatus {
        match &self.status {
            FireStatus::Miss => {
                let status = pedersen_hash(&Felt::ZERO, &salt.into());
                SaltedFireStatus::Miss(status)
            },
            FireStatus::Hit(kind) => {
                let id = Felt::from(kind.id());
                let status = pedersen_hash(&id, &salt.into());
                SaltedFireStatus::Hit(((*kind).into(), status))
            }
        }
    }
}

impl Display for FireReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut report = String::new();

        match self.status {
            FireStatus::Miss => report.push_str("MISS."),
            FireStatus::Hit(kind) => report.push_str(format!("HIT {}", kind).as_str()),
        }

        if let Some(ship) = self.ship_destroyed {
            report.push_str(" ");
            report.push_str(format!("{} destroyed!", ship.kind).as_str());
        }

        writeln!(f, "{}", report)
    }
}