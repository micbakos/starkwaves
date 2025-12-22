use std::collections::HashSet;
use crate::types::ShipKind;
use derive_more::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum BoardSize {
    // 10x10
    #[display("10x10")]
    Standard,

    // 6x6, 8x8
    Smaller(SmallerBoardSize),

    // 12x12, 14x14, 20x20
    Larger(LargerBoardSize),
}

impl BoardSize {
    pub fn size(&self) -> usize {
        match self {
            BoardSize::Standard => 10,
            BoardSize::Smaller(smaller) => smaller.size(),
            BoardSize::Larger(larger) => larger.size(),
        }
    }

    pub fn eligible_ship_kinds(&self) -> HashSet<ShipKind> {
        match self {
            BoardSize::Standard => ShipKind::all().difference(
                &HashSet::from([ShipKind::SuperCarrier])
            ).cloned().collect(),
            BoardSize::Smaller(_) => HashSet::from([ShipKind::Cruiser, ShipKind::Destroyer]),
            BoardSize::Larger(_) => ShipKind::all()
        }
    }

    pub fn ship_kinds_count(&self, ship_kind: &ShipKind) -> usize {
        let eligible = self.eligible_ship_kinds();
        match self {
            BoardSize::Standard => if eligible.contains(ship_kind) { 1 } else { 0 },
            BoardSize::Smaller(_) => if eligible.contains(ship_kind) { 1 } else { 0 },
            BoardSize::Larger(_) => {
                if ship_kind == &ShipKind::Destroyer || ship_kind == &ShipKind::Submarine {
                    2
                } else if eligible.contains(ship_kind) {
                    1
                } else {
                    0
                }
            }
        }
    }
}

impl Default for BoardSize {
    fn default() -> Self {
        BoardSize::Standard
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum SmallerBoardSize {
    #[display("6x6")]
    SixBySix,
    #[display("8x8")]
    EightByEight,
}

impl SmallerBoardSize {
    pub fn size(&self) -> usize {
        match self {
            SmallerBoardSize::SixBySix => 6,
            SmallerBoardSize::EightByEight => 8
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum LargerBoardSize {
    #[display("12x12")]
    TwelveByTwelve,
    #[display("14x14")]
    FourteenByFourteen,
    #[display("20x20")]
    TwentyByTwenty,
}

impl LargerBoardSize {
    pub fn size(&self) -> usize {
        match self {
            LargerBoardSize::TwelveByTwelve => 12,
            LargerBoardSize::FourteenByFourteen => 14,
            LargerBoardSize::TwentyByTwenty => 20
        }
    }
}