use crate::cairo::cairo_value::CairoValue;
use cairo_native::Value;
use derive_more::Display;
use starknet::core::types::Felt;

/// Represents the orientation of a ship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Orientation {
    #[display("horizontal")]
    Horizontal,
    #[display("vertical")]
    Vertical,
}

impl Into<CairoValue> for Orientation {
    fn into(self) -> CairoValue {
        let orientation_tag = match self {
            Orientation::Horizontal => 0,
            Orientation::Vertical => 1,
        };

        CairoValue(Value::Felt252(Felt::from(orientation_tag)))
    }
}