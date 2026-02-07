use core::fmt::Display;

#[derive(Debug, Drop, Serde, Copy)]
pub struct Ship {
    pub kind: ShipKind,
    pub x: u8,
    pub y: u8,
    pub orientation: Orientation,
}

#[derive(Debug, Drop, Serde, Copy)]
pub enum Orientation {
    Horizontal,
    Vertical,
}


#[derive(Debug, Drop, PartialEq, Copy, Serde)]
pub enum ShipKind {
    Carrier, // Size 5
    Battleship, // Size 4
    Cruiser, // Size 3
    Submarine, // Size 3
    Destroyer, // Size 2
    SuperCarrier // Size 6
}

impl ShipKindDisplay of Display<ShipKind> {
    fn fmt(self: @ShipKind, ref f: core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        let rep: ByteArray = match self {
            ShipKind::Carrier => "CA",
            ShipKind::Battleship => "BA",
            ShipKind::Cruiser => "CR",
            ShipKind::Submarine => "SU",
            ShipKind::Destroyer => "DE",
            ShipKind::SuperCarrier => "SC",
        };
        write!(f, "{rep}")
    }
}

#[generate_trait]
pub impl ShipKindImpl of ShipKindTrait {
    fn id(self: @ShipKind) -> u8 {
        match self {
            ShipKind::Carrier => 1,
            ShipKind::Battleship => 2,
            ShipKind::Cruiser => 3,
            ShipKind::Submarine => 4,
            ShipKind::Destroyer => 5,
            ShipKind::SuperCarrier => 6,
        }
    }

    fn length(self: @ShipKind) -> u8 {
        match self {
            ShipKind::SuperCarrier => 6,
            ShipKind::Carrier => 5,
            ShipKind::Battleship => 4,
            ShipKind::Cruiser => 3,
            ShipKind::Submarine => 3,
            ShipKind::Destroyer => 2,
        }
    }

    fn is_eligible(self: ShipKind, board_size: u8, occurences: u8) -> bool {
        if (board_size == 6 || board_size == 8) {
            if (self == ShipKind::Cruiser || self == ShipKind::Destroyer) {
                return occurences == 1;
            } else {
                return occurences == 0;
            }
        } else if (board_size == 10) {
            if (self != ShipKind::SuperCarrier) {
                return occurences == 1;
            } else {
                return occurences == 0;
            }
        } else if (board_size == 12 || board_size == 14 || board_size == 20) {
            if (self == ShipKind::Destroyer || self == ShipKind::Submarine) {
                return occurences == 2;
            } else {
                return occurences == 1;
            }
        } else {
            return false;
        }
    }

    fn all() -> Array<ShipKind> {
        array![
            ShipKind::Carrier, ShipKind::Battleship, ShipKind::Cruiser, ShipKind::Submarine,
            ShipKind::Destroyer, ShipKind::SuperCarrier,
        ]
    }
}
