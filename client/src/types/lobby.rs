use crate::types::board_size::{BoardSize, LargerBoardSize, SmallerBoardSize};
use cainome::cairo_serde::ContractAddress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lobbies {
    six: Option<ContractAddress>,
    eight: Option<ContractAddress>,
    ten: Option<ContractAddress>,
    twelve: Option<ContractAddress>,
    fourteen: Option<ContractAddress>,
    twenty: Option<ContractAddress>,
}

impl Lobbies {
    
    pub fn new(waitlist: Vec<(BoardSize, ContractAddress)>) -> Self {
        let mut lobbies = Lobbies {
            six: None,
            eight: None,
            ten: None,
            twelve: None,
            fourteen: None,
            twenty: None,
        };
        
        waitlist.iter().for_each(|(board_size, address)| {
            match board_size {
                BoardSize::Standard => {
                    lobbies.ten = Some(*address);
                }
                BoardSize::Smaller(smaller) => match smaller {
                    SmallerBoardSize::SixBySix => lobbies.six = Some(*address),
                    SmallerBoardSize::EightByEight => lobbies.eight = Some(*address),
                }
                BoardSize::Larger(larger) => match larger {
                    LargerBoardSize::TwelveByTwelve => lobbies.twelve = Some(*address),
                    LargerBoardSize::FourteenByFourteen => lobbies.fourteen = Some(*address),
                    LargerBoardSize::TwentyByTwenty => lobbies.twenty = Some(*address),
                }
            }
        });
        
        lobbies
    }
    
    pub fn lobby(&self, size: BoardSize) -> Option<ContractAddress> {
        match size {
            BoardSize::Standard => self.ten,
            BoardSize::Smaller(smaller) => match smaller {
                SmallerBoardSize::SixBySix => self.six,
                SmallerBoardSize::EightByEight => self.eight
            }
            BoardSize::Larger(larger) => match larger {
                LargerBoardSize::TwelveByTwelve => self.twelve,
                LargerBoardSize::FourteenByFourteen => self.fourteen,
                LargerBoardSize::TwentyByTwenty => self.twenty,
            }
        }
    }
}