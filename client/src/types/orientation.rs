use derive_more::Display;

/// Represents the orientation of a ship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum Orientation {
    #[display("horizontal")]
    Horizontal,
    #[display("vertical")]
    Vertical,
}