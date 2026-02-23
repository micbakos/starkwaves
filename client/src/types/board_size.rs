use crate::types::ShipKind;
use derive_more::Display;
use std::collections::HashSet;
use starknet::core::codec::Encode;
use crate::types::contract::starkwaves;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Encode)]
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
    pub fn size(&self) -> u8 {
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

impl Into<starkwaves::BoardSize> for BoardSize {
    fn into(self) -> starkwaves::BoardSize {
        match self {
            BoardSize::Standard => starkwaves::BoardSize::Standard,
            BoardSize::Smaller(smaller) => starkwaves::BoardSize::Smaller(smaller.into()),
            BoardSize::Larger(larger) => starkwaves::BoardSize::Larger(larger.into()),
        }
    }
}

impl From<starkwaves::BoardSize> for BoardSize {
    fn from(value: starkwaves::BoardSize) -> Self {
        match value {
            starkwaves::BoardSize::Standard => BoardSize::Standard,
            starkwaves::BoardSize::Smaller(smaller) => BoardSize::Smaller(smaller.into()),
            starkwaves::BoardSize::Larger(larger) => BoardSize::Larger(larger.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Encode)]
pub enum SmallerBoardSize {
    #[display("6x6")]
    SixBySix,
    #[display("8x8")]
    EightByEight,
}

impl SmallerBoardSize {
    pub fn size(&self) -> u8 {
        match self {
            SmallerBoardSize::SixBySix => 6,
            SmallerBoardSize::EightByEight => 8
        }
    }
}

impl Into<starkwaves::SmallerBoardSize> for SmallerBoardSize {
    fn into(self) -> starkwaves::SmallerBoardSize {
        match self {
            SmallerBoardSize::SixBySix => starkwaves::SmallerBoardSize::SixBySix,
            SmallerBoardSize::EightByEight => starkwaves::SmallerBoardSize::EightByEight,
        }
    }
}

impl From<starkwaves::SmallerBoardSize> for SmallerBoardSize {
    fn from(value: starkwaves::SmallerBoardSize) -> Self {
        match value {
            starkwaves::SmallerBoardSize::SixBySix => SmallerBoardSize::SixBySix,
            starkwaves::SmallerBoardSize::EightByEight => SmallerBoardSize::EightByEight,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Encode)]
pub enum LargerBoardSize {
    #[display("12x12")]
    TwelveByTwelve,
    #[display("14x14")]
    FourteenByFourteen,
    #[display("20x20")]
    TwentyByTwenty,
}

impl LargerBoardSize {
    pub fn size(&self) -> u8 {
        match self {
            LargerBoardSize::TwelveByTwelve => 12,
            LargerBoardSize::FourteenByFourteen => 14,
            LargerBoardSize::TwentyByTwenty => 20
        }
    }
}

impl Into<starkwaves::LargerBoardSize> for LargerBoardSize {
    fn into(self) -> starkwaves::LargerBoardSize {
        match self {
            LargerBoardSize::TwelveByTwelve => starkwaves::LargerBoardSize::TwelveByTwelve,
            LargerBoardSize::FourteenByFourteen => starkwaves::LargerBoardSize::FourteenByFourteen,
            LargerBoardSize::TwentyByTwenty => starkwaves::LargerBoardSize::TwentyByTwenty,
        }
    }
}

impl From<starkwaves::LargerBoardSize> for LargerBoardSize {
    fn from(value: starkwaves::LargerBoardSize) -> Self {
        match value {
            starkwaves::LargerBoardSize::TwelveByTwelve => LargerBoardSize::TwelveByTwelve,
            starkwaves::LargerBoardSize::FourteenByFourteen => LargerBoardSize::FourteenByFourteen,
            starkwaves::LargerBoardSize::TwentyByTwenty => LargerBoardSize::TwentyByTwenty
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // BoardSize tests
    #[test]
    fn test_board_size_standard() {
        let board = BoardSize::Standard;
        assert_eq!(board.size(), 10);
        assert_eq!(format!("{}", board), "10x10");
    }

    #[test]
    fn test_board_size_smaller_6x6() {
        let board = BoardSize::Smaller(SmallerBoardSize::SixBySix);
        assert_eq!(board.size(), 6);
        assert_eq!(format!("{}", board), "6x6");
    }

    #[test]
    fn test_board_size_smaller_8x8() {
        let board = BoardSize::Smaller(SmallerBoardSize::EightByEight);
        assert_eq!(board.size(), 8);
        assert_eq!(format!("{}", board), "8x8");
    }

    #[test]
    fn test_board_size_larger_12x12() {
        let board = BoardSize::Larger(LargerBoardSize::TwelveByTwelve);
        assert_eq!(board.size(), 12);
        assert_eq!(format!("{}", board), "12x12");
    }

    #[test]
    fn test_board_size_larger_14x14() {
        let board = BoardSize::Larger(LargerBoardSize::FourteenByFourteen);
        assert_eq!(board.size(), 14);
        assert_eq!(format!("{}", board), "14x14");
    }

    #[test]
    fn test_board_size_larger_20x20() {
        let board = BoardSize::Larger(LargerBoardSize::TwentyByTwenty);
        assert_eq!(board.size(), 20);
        assert_eq!(format!("{}", board), "20x20");
    }

    #[test]
    fn test_board_size_default() {
        let board = BoardSize::default();
        assert_eq!(board, BoardSize::Standard);
        assert_eq!(board.size(), 10);
    }

    // Eligible ships tests
    #[test]
    fn test_standard_board_eligible_ships() {
        let board = BoardSize::Standard;
        let eligible = board.eligible_ship_kinds();

        assert_eq!(eligible.len(), 5);
        assert!(eligible.contains(&ShipKind::Carrier));
        assert!(eligible.contains(&ShipKind::Battleship));
        assert!(eligible.contains(&ShipKind::Cruiser));
        assert!(eligible.contains(&ShipKind::Submarine));
        assert!(eligible.contains(&ShipKind::Destroyer));
        assert!(
            !eligible.contains(&ShipKind::SuperCarrier),
            "SuperCarrier should not be eligible on Standard board"
        );
    }

    #[test]
    fn test_smaller_board_eligible_ships() {
        let board = BoardSize::Smaller(SmallerBoardSize::SixBySix);
        let eligible = board.eligible_ship_kinds();

        assert_eq!(eligible.len(), 2);
        assert!(eligible.contains(&ShipKind::Cruiser));
        assert!(eligible.contains(&ShipKind::Destroyer));
        assert!(!eligible.contains(&ShipKind::SuperCarrier));
        assert!(!eligible.contains(&ShipKind::Carrier));
        assert!(!eligible.contains(&ShipKind::Battleship));
    }

    #[test]
    fn test_larger_board_eligible_ships() {
        let board = BoardSize::Larger(LargerBoardSize::TwelveByTwelve);
        let eligible = board.eligible_ship_kinds();

        assert_eq!(eligible.len(), 6, "All ships should be eligible on larger boards");
        assert!(eligible.contains(&ShipKind::SuperCarrier));
        assert!(eligible.contains(&ShipKind::Carrier));
        assert!(eligible.contains(&ShipKind::Battleship));
        assert!(eligible.contains(&ShipKind::Cruiser));
        assert!(eligible.contains(&ShipKind::Submarine));
        assert!(eligible.contains(&ShipKind::Destroyer));
    }

    // Ship count tests
    #[test]
    fn test_standard_board_ship_counts() {
        let board = BoardSize::Standard;

        assert_eq!(board.ship_kinds_count(&ShipKind::Carrier), 1);
        assert_eq!(board.ship_kinds_count(&ShipKind::Battleship), 1);
        assert_eq!(board.ship_kinds_count(&ShipKind::Cruiser), 1);
        assert_eq!(board.ship_kinds_count(&ShipKind::Submarine), 1);
        assert_eq!(board.ship_kinds_count(&ShipKind::Destroyer), 1);
        assert_eq!(
            board.ship_kinds_count(&ShipKind::SuperCarrier), 0,
            "SuperCarrier not allowed on Standard board"
        );
    }

    #[test]
    fn test_smaller_board_ship_counts() {
        let board = BoardSize::Smaller(SmallerBoardSize::EightByEight);

        assert_eq!(board.ship_kinds_count(&ShipKind::Cruiser), 1);
        assert_eq!(board.ship_kinds_count(&ShipKind::Destroyer), 1);
        assert_eq!(board.ship_kinds_count(&ShipKind::Carrier), 0);
        assert_eq!(board.ship_kinds_count(&ShipKind::Battleship), 0);
        assert_eq!(board.ship_kinds_count(&ShipKind::Submarine), 0);
        assert_eq!(board.ship_kinds_count(&ShipKind::SuperCarrier), 0);
    }

    #[test]
    fn test_larger_board_ship_counts() {
        let board = BoardSize::Larger(LargerBoardSize::TwelveByTwelve);

        // Destroyer and Submarine get 2 each on larger boards
        assert_eq!(board.ship_kinds_count(&ShipKind::Destroyer), 2);
        assert_eq!(board.ship_kinds_count(&ShipKind::Submarine), 2);

        // Others get 1 each
        assert_eq!(board.ship_kinds_count(&ShipKind::SuperCarrier), 1);
        assert_eq!(board.ship_kinds_count(&ShipKind::Carrier), 1);
        assert_eq!(board.ship_kinds_count(&ShipKind::Battleship), 1);
        assert_eq!(board.ship_kinds_count(&ShipKind::Cruiser), 1);
    }

    // SmallerBoardSize tests
    #[test]
    fn test_smaller_board_sizes() {
        assert_eq!(SmallerBoardSize::SixBySix.size(), 6);
        assert_eq!(SmallerBoardSize::EightByEight.size(), 8);
    }

    #[test]
    fn test_smaller_board_display() {
        assert_eq!(format!("{}", SmallerBoardSize::SixBySix), "6x6");
        assert_eq!(format!("{}", SmallerBoardSize::EightByEight), "8x8");
    }

    #[test]
    fn test_smaller_board_equality() {
        assert_eq!(SmallerBoardSize::SixBySix, SmallerBoardSize::SixBySix);
        assert_ne!(SmallerBoardSize::SixBySix, SmallerBoardSize::EightByEight);
    }

    // LargerBoardSize tests
    #[test]
    fn test_larger_board_sizes() {
        assert_eq!(LargerBoardSize::TwelveByTwelve.size(), 12);
        assert_eq!(LargerBoardSize::FourteenByFourteen.size(), 14);
        assert_eq!(LargerBoardSize::TwentyByTwenty.size(), 20);
    }

    #[test]
    fn test_larger_board_display() {
        assert_eq!(format!("{}", LargerBoardSize::TwelveByTwelve), "12x12");
        assert_eq!(format!("{}", LargerBoardSize::FourteenByFourteen), "14x14");
        assert_eq!(format!("{}", LargerBoardSize::TwentyByTwenty), "20x20");
    }

    #[test]
    fn test_larger_board_equality() {
        assert_eq!(LargerBoardSize::TwelveByTwelve, LargerBoardSize::TwelveByTwelve);
        assert_ne!(LargerBoardSize::TwelveByTwelve, LargerBoardSize::FourteenByFourteen);
    }

    // BoardSize equality and copy tests
    #[test]
    fn test_board_size_equality() {
        assert_eq!(BoardSize::Standard, BoardSize::Standard);
        assert_ne!(BoardSize::Standard, BoardSize::Smaller(SmallerBoardSize::SixBySix));

        assert_eq!(
            BoardSize::Smaller(SmallerBoardSize::SixBySix),
            BoardSize::Smaller(SmallerBoardSize::SixBySix)
        );

        assert_eq!(
            BoardSize::Larger(LargerBoardSize::TwelveByTwelve),
            BoardSize::Larger(LargerBoardSize::TwelveByTwelve)
        );
    }

    #[test]
    fn test_board_size_copy() {
        let board = BoardSize::Standard;
        let copied = board;
        assert_eq!(board, copied);
        assert_eq!(board.size(), 10); // Original still usable
    }

    #[test]
    fn test_board_size_clone() {
        let board = BoardSize::Larger(LargerBoardSize::TwentyByTwenty);
        let cloned = board.clone();
        assert_eq!(board, cloned);
        assert_eq!(cloned.size(), 20);
    }
}