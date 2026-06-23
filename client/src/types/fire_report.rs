use crate::types::{Ship, ShipKind};
use starknet_rust::core::crypto::pedersen_hash;
use starknet_rust::core::types::Felt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct FireReport {
    pub status: FireStatus,
    pub ship_destroyed: Option<Ship>,
    pub proof: Vec<Felt>,
}

impl FireReport {
    pub fn miss(proof: Vec<Felt>) -> FireReport {
        FireReport {
            status: FireStatus::Miss,
            ship_destroyed: None,
            proof,
        }
    }

    pub fn hit(kind: ShipKind, proof: Vec<Felt>) -> FireReport {
        FireReport {
            status: FireStatus::Hit(kind),
            ship_destroyed: None,
            proof,
        }
    }

    pub fn hit_with_destruction(ship: Ship, proof: Vec<Felt>) -> FireReport {
        FireReport {
            status: FireStatus::Hit(ship.kind),
            ship_destroyed: Some(ship),
            proof,
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

#[derive(Debug, PartialEq, Eq)]
pub enum FireStatus {
    Miss,
    Hit(ShipKind),
}

impl FireStatus {
    pub fn salted(&self, salt: u64) -> Felt {
        let status = match &self {
            FireStatus::Miss => 0,
            FireStatus::Hit(_) => 1,
        };
        pedersen_hash(&Felt::from(status), &Felt::from(salt))
    }
}
