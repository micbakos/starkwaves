use enum_as_inner::EnumAsInner;
use starkwaves_client::types::lobby::Lobbies;
use strum::VariantArray;
use starkwaves_client::types::board_size::{BoardSize, LargerBoardSize, SmallerBoardSize};
use crate::app::types::LoggedAccount;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub lobby: LobbyState,
    pub selected_lobby: Option<LobbyVariant>,
    pub selected_account_menu_item: Option<AccountMenu>,
    pub account: LoggedAccount
}

impl State {
    pub fn new(account: LoggedAccount) -> Self {
        Self {
            lobby: LobbyState::Idle,
            selected_lobby: None,
            selected_account_menu_item: None,
            account
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum LobbyState {
    Idle,
    Resolving,
    Received(Lobbies)
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, VariantArray)]
pub enum LobbyVariant {
    Six,
    Eight,
    Ten,
    Twelve,
    Fourteen,
    Twenty
}

impl From<LobbyVariant> for BoardSize {
    fn from(value: LobbyVariant) -> Self {
        match value {
            LobbyVariant::Six => Self::Smaller(SmallerBoardSize::SixBySix),
            LobbyVariant::Eight => Self::Smaller(SmallerBoardSize::EightByEight),
            LobbyVariant::Ten => Self::Standard,
            LobbyVariant::Twelve => Self::Larger(LargerBoardSize::TwelveByTwelve),
            LobbyVariant::Fourteen => Self::Larger(LargerBoardSize::FourteenByFourteen),
            LobbyVariant::Twenty => Self::Larger(LargerBoardSize::TwentyByTwenty)
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, VariantArray)]
pub enum AccountMenu {
    Copy,
    Logout
}

pub enum Intent {
    OnStart,
    OnUpdateLobbyState(LobbyState),
    OnSelectPreviousLobby,
    OnSelectNextLobby,
    OnMoveFocusToAccount,
    OnMoveFocusToLobby,
    OnSelectNextAccountMenuItem,
    OnSelectPrevAccountMenuItem,
    OnSelectionClicked
}

pub enum Effect {
    RequestGetLobbies,
    RequestCopyToClipboard(String),
    RequestLogout,
}