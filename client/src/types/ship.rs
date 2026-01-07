use std::vec::IntoIter;
use crate::cairo::cairo_value::CairoValue;
use crate::types::{Orientation, ShipKind};
use cairo_native::Value;
use starknet::core::types::Felt;

/// Represents a ship placed on the board
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ship {
    pub kind: ShipKind,
    pub x: u8,
    pub y: u8,
    pub orientation: Orientation,
}

impl Ship {
    pub fn new(kind: ShipKind, x: u8, y: u8, orientation: Orientation) -> Ship {
        Ship {
            kind,
            x,
            y,
            orientation,
        }
    }
}

impl Into<CairoValue> for Ship {
    fn into(self) -> CairoValue {
        let x = self.x;
        let y = self.y;

        let kind_value: CairoValue = self.kind.into();
        let orientation_value: CairoValue = self.orientation.into();

        CairoValue(Value::Struct {
            fields: vec![
                kind_value.0,
                Value::Felt252(Felt::from(x)),
                Value::Felt252(Felt::from(y)),
                orientation_value.0,
            ],
            debug_name: Some("Ship".to_string()),
        })
    }
}