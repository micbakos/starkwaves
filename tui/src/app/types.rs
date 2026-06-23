use crate::onboard::start;
use crate::types::ScreenState;
use enum_as_inner::EnumAsInner;
use starknet_rust_core::types::Felt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub core: CoreState,
    pub screens: Vec<ScreenState>,
}

impl AppState {
    pub fn start() -> Self {
        Self {
            core: CoreState {
                account: AccountState::None,
                toast: None,
                running: true,
            },
            screens: vec![start::types::State::new().into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreState {
    pub account: AccountState,
    pub toast: Option<String>,
    pub running: bool,
}

pub enum Effect {
    RequestQuit,
    RequestNavigateTo(ScreenState),
    RequestNavigateBack,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum Intent {
    Quit,
    Open(ScreenState),
    GoBack,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum AccountState {
    None,
    LoggedIn { address: Felt, kind: AccountKind },
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum AccountKind {
    Local,
    Cartridge,
}
