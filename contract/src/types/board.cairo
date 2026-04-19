use core::fmt::Display;
use crate::Felt252Dict;
use super::ShipKind;
use super::ship::{Orientation, Ship, ShipKindTrait};

pub trait BoardSizeTrait<B> {
    fn size(self: @B) -> u8;

    fn total_hits(self: @B) -> u8;
}

#[derive(Debug, Copy, PartialEq, Drop, Serde, starknet::Store)]
pub enum BoardSize {
    #[default]
    // 10x10
    Standard,
    // 6x6, 8x8
    Smaller: SmallerBoardSize,
    // 12x12, 14x14, 20x20
    Larger: LargerBoardSize,
}

impl BoardSizeDefault of Default<BoardSize> {
    fn default() -> BoardSize {
        BoardSize::Standard
    }
}

impl DisplayImpl of Display<BoardSize> {
    fn fmt(self: @BoardSize, ref f: core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        let display: ByteArray = match self {
            BoardSize::Standard => "10x10",
            BoardSize::Smaller(smaller) => format!("{}", *smaller),
            BoardSize::Larger(larger) => format!("{}", *larger),
        };
        write!(f, "{display}")
    }
}


#[generate_trait]
pub impl AllBoardSizesImpl of AllBoardSizesTrait {
    fn all() -> Span<BoardSize> {
        array![
            BoardSize::Standard, BoardSize::Smaller(SmallerBoardSize::SixBySix),
            BoardSize::Smaller(SmallerBoardSize::EightByEight),
            BoardSize::Larger(LargerBoardSize::TwelveByTwelve),
            BoardSize::Larger(LargerBoardSize::FourteenByFourteen),
            BoardSize::Larger(LargerBoardSize::TwentyByTwenty),
        ]
            .span()
    }
}

pub impl BoardSizeIntoSize of BoardSizeTrait<BoardSize> {
    fn size(self: @BoardSize) -> u8 {
        match self {
            BoardSize::Standard => 10,
            BoardSize::Smaller(smaller) => smaller.size(),
            BoardSize::Larger(larger) => larger.size(),
        }
    }

    fn total_hits(self: @BoardSize) -> u8 {
        match self {
            BoardSize::Standard => ShipKind::Carrier.length()
                + ShipKind::Battleship.length()
                + ShipKind::Cruiser.length()
                + ShipKind::Submarine.length()
                + ShipKind::Destroyer.length(),
            BoardSize::Smaller(smaller) => smaller.total_hits(),
            BoardSize::Larger(larger) => larger.total_hits(),
        }
    }
}

#[derive(Debug, Copy, PartialEq, Drop, Serde, starknet::Store)]
pub enum SmallerBoardSize {
    #[default]
    SixBySix,
    EightByEight,
}

impl SmallerDisplayImpl of Display<SmallerBoardSize> {
    fn fmt(self: @SmallerBoardSize, ref f: core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        let display: ByteArray = match self {
            SmallerBoardSize::SixBySix => "6x6",
            SmallerBoardSize::EightByEight => "8x8",
        };
        write!(f, "{display}")
    }
}

pub impl SmallerBoardSizeImpl of BoardSizeTrait<SmallerBoardSize> {
    fn size(self: @SmallerBoardSize) -> u8 {
        match self {
            SmallerBoardSize::SixBySix => 6,
            SmallerBoardSize::EightByEight => 8,
        }
    }

    fn total_hits(self: @SmallerBoardSize) -> u8 {
        ShipKind::Cruiser.length() + ShipKind::Destroyer.length()
    }
}

#[derive(Debug, Copy, PartialEq, Drop, Serde, starknet::Store)]
pub enum LargerBoardSize {
    #[default]
    TwelveByTwelve,
    FourteenByFourteen,
    TwentyByTwenty,
}

impl LargerDisplayImpl of Display<LargerBoardSize> {
    fn fmt(self: @LargerBoardSize, ref f: core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        let display: ByteArray = match self {
            LargerBoardSize::TwelveByTwelve => "12x12",
            LargerBoardSize::FourteenByFourteen => "14x14",
            LargerBoardSize::TwentyByTwenty => "20x20",
        };
        write!(f, "{display}")
    }
}

pub impl LargerBoardSizeImpl of BoardSizeTrait<LargerBoardSize> {
    fn size(self: @LargerBoardSize) -> u8 {
        match self {
            LargerBoardSize::TwelveByTwelve => 12,
            LargerBoardSize::FourteenByFourteen => 14,
            LargerBoardSize::TwentyByTwenty => 20,
        }
    }

    fn total_hits(self: @LargerBoardSize) -> u8 {
        ShipKind::SuperCarrier.length()
            + ShipKind::Carrier.length()
            + ShipKind::Battleship.length()
            + ShipKind::Cruiser.length()
            + ShipKind::Submarine.length() * 2
            + ShipKind::Destroyer.length() * 2
    }
}

pub fn create_board(ships: Span<Ship>, board_size: u8) -> Array<bool> {
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

    let mut board_array: Array<bool> = ArrayTrait::new();
    let array_size: u32 = board_size.into() * board_size.into();
    for i in 0..array_size {
        let ship_id = board.get(i.into());
        board_array.append(ship_id != 0);
    }

    return board_array;
}
