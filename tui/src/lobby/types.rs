use crate::app::services::OnChainData;
use crate::app::types::LoggedAccount;
use crate::types::result::Result;
use enum_as_inner::EnumAsInner;
use starkwaves_client::types::board_size::{BoardSize, LargerBoardSize, SmallerBoardSize};
use starkwaves_client::types::lobby::Lobbies;
use strum::{EnumMessage, VariantArray};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub lobby: LobbyState,
    pub selected_lobby: Option<LobbyVariant>,
    pub selected_account_menu_item: Option<AccountMenu>,
    pub exit_lobby_popup: Option<ExitLobbyPopup>,
    pub account: LoggedAccount,
}

impl State {
    pub fn new(account: LoggedAccount) -> Self {
        Self {
            lobby: LobbyState::Idle,
            selected_lobby: None,
            selected_account_menu_item: None,
            exit_lobby_popup: None,
            account,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum LobbyState {
    Idle,
    Resolving,
    Received(Lobbies),
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, VariantArray)]
pub enum LobbyVariant {
    Six,
    Eight,
    Ten,
    Twelve,
    Fourteen,
    Twenty,
}

impl From<LobbyVariant> for BoardSize {
    fn from(value: LobbyVariant) -> Self {
        match value {
            LobbyVariant::Six => Self::Smaller(SmallerBoardSize::SixBySix),
            LobbyVariant::Eight => Self::Smaller(SmallerBoardSize::EightByEight),
            LobbyVariant::Ten => Self::Standard,
            LobbyVariant::Twelve => Self::Larger(LargerBoardSize::TwelveByTwelve),
            LobbyVariant::Fourteen => Self::Larger(LargerBoardSize::FourteenByFourteen),
            LobbyVariant::Twenty => Self::Larger(LargerBoardSize::TwentyByTwenty),
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, VariantArray)]
pub enum AccountMenu {
    Copy,
    Logout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitLobbyPopup {
    pub lobby_size: BoardSize,
    pub selected_action: ExitLobbyPopupAction,
}

impl ExitLobbyPopup {
    pub fn new(lobby_size: BoardSize) -> Self {
        Self {
            lobby_size,
            selected_action: ExitLobbyPopupAction::Exit,
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, VariantArray, EnumMessage)]
pub enum ExitLobbyPopupAction {
    #[strum(message = "Exit")]
    Exit,
    #[strum(message = "Cancel")]
    Cancel,
}

pub enum Intent {
    OnTimeToRefreshLobbyState,
    OnUpdateLobbyState(LobbyState),
    OnJoinedLobby(BoardSize),
    OnExitedLobby(BoardSize),
    OnSelectPreviousLobby,
    OnSelectNextLobby,
    OnMoveFocusToAccount,
    OnMoveFocusToLobby,
    OnSelectNextAccountMenuItem,
    OnSelectPrevAccountMenuItem,
    OnSelectNextExitLobbyPopupMenuItem,
    OnSelectPrevExitLobbyPopupMenuItem,
    OnSelectionClicked,
}

pub enum Effect {
    RequestGetLobbies,
    RequestJoinLobby(LobbyVariant),
    RequestExitLobby(BoardSize),
    RequestCopyToClipboard(String),
    RequestLogout,
}
