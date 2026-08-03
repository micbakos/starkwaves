use starknet_rust_core::types::Felt;

use crate::types::board_size::{BoardSize, LargerBoardSize, SmallerBoardSize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lobbies {
    six: Option<Felt>,
    eight: Option<Felt>,
    ten: Option<Felt>,
    twelve: Option<Felt>,
    fourteen: Option<Felt>,
    twenty: Option<Felt>,
}

impl Lobbies {
    pub fn new(waitlist: Vec<(BoardSize, impl Into<Felt>)>) -> Self {
        let mut lobbies = Lobbies {
            six: None,
            eight: None,
            ten: None,
            twelve: None,
            fourteen: None,
            twenty: None,
        };

        waitlist
            .into_iter()
            .for_each(|(board_size, address)| match board_size {
                BoardSize::Standard => {
                    lobbies.ten = Some(address.into());
                }
                BoardSize::Smaller(smaller) => match smaller {
                    SmallerBoardSize::SixBySix => lobbies.six = Some(address.into()),
                    SmallerBoardSize::EightByEight => lobbies.eight = Some(address.into()),
                },
                BoardSize::Larger(larger) => match larger {
                    LargerBoardSize::TwelveByTwelve => lobbies.twelve = Some(address.into()),
                    LargerBoardSize::FourteenByFourteen => lobbies.fourteen = Some(address.into()),
                    LargerBoardSize::TwentyByTwenty => lobbies.twenty = Some(address.into()),
                },
            });

        lobbies
    }

    pub fn lobby(&self, size: BoardSize) -> Option<Felt> {
        match size {
            BoardSize::Standard => self.ten,
            BoardSize::Smaller(smaller) => match smaller {
                SmallerBoardSize::SixBySix => self.six,
                SmallerBoardSize::EightByEight => self.eight,
            },
            BoardSize::Larger(larger) => match larger {
                LargerBoardSize::TwelveByTwelve => self.twelve,
                LargerBoardSize::FourteenByFourteen => self.fourteen,
                LargerBoardSize::TwentyByTwenty => self.twenty,
            },
        }
    }

    pub fn join(&mut self, size: BoardSize, address: impl Into<Felt>) {
        match size {
            BoardSize::Standard => self.ten = Some(address.into()),
            BoardSize::Smaller(smaller_board_size) => match smaller_board_size {
                SmallerBoardSize::SixBySix => self.six = Some(address.into()),
                SmallerBoardSize::EightByEight => self.eight = Some(address.into()),
            },
            BoardSize::Larger(larger_board_size) => match larger_board_size {
                LargerBoardSize::TwelveByTwelve => self.twelve = Some(address.into()),
                LargerBoardSize::FourteenByFourteen => self.fourteen = Some(address.into()),
                LargerBoardSize::TwentyByTwenty => self.twenty = Some(address.into()),
            },
        }
    }

    pub fn exit(&mut self, size: BoardSize) {
        match size {
            BoardSize::Standard => self.ten = None,
            BoardSize::Smaller(smaller_board_size) => match smaller_board_size {
                SmallerBoardSize::SixBySix => self.six = None,
                SmallerBoardSize::EightByEight => self.eight = None,
            },
            BoardSize::Larger(larger_board_size) => match larger_board_size {
                LargerBoardSize::TwelveByTwelve => self.twelve = None,
                LargerBoardSize::FourteenByFourteen => self.fourteen = None,
                LargerBoardSize::TwentyByTwenty => self.twenty = None,
            },
        }
    }

    pub fn player_lobby(&self, address: impl Into<Felt>) -> Option<BoardSize> {
        let address: Felt = address.into();
        if self.six == Some(address) {
            Some(BoardSize::Smaller(SmallerBoardSize::SixBySix))
        } else if self.eight == Some(address) {
            Some(BoardSize::Smaller(SmallerBoardSize::EightByEight))
        } else if self.ten == Some(address) {
            Some(BoardSize::Standard)
        } else if self.twelve == Some(address) {
            Some(BoardSize::Larger(LargerBoardSize::TwelveByTwelve))
        } else if self.fourteen == Some(address) {
            Some(BoardSize::Larger(LargerBoardSize::FourteenByFourteen))
        } else if self.twenty == Some(address) {
            Some(BoardSize::Larger(LargerBoardSize::TwentyByTwenty))
        } else {
            None
        }
    }
}
