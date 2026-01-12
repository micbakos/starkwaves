use core::fmt::Display;
use crate::Felt252Dict;

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

#[derive(Debug, Drop, Serde, Copy)]
pub enum FireStatus {
    Miss,
    Hit: ShipKind,
}

pub fn board(ships: Span<Ship>, board_size: u8) -> Array<u8> {
    let mut board: Felt252Dict<u8> = Default::default();

    let offset = |x: u8, y: u8| -> felt252 {
        let rows_offset: u32 = x.into() * board_size.into();
        (rows_offset + y.into()).into()
    };

    for ship in ships {
        let id = ship.kind.id();
        let size = ship.kind.length();

        for step in 0..size {
            let (x, y) = match ship.orientation {
                Orientation::Horizontal => (*ship.x, *ship.y + step),
                Orientation::Vertical => (*ship.x + step, *ship.y),
            };

            let offset = offset(x, y);
            let item = board.get(offset);

            assert!(item == 0, "Ship {} collides with {} in [{},{}]", id, item, x, y)

            board.insert(offset, id);
        }
    }

    let mut board_array: Array<u8> = ArrayTrait::new();
    let array_size: u32 = board_size.into() * board_size.into();
    for i in 0..array_size {
        board_array.append(board.get(i.into()));
    }

    return board_array;
}
