use crate::types::ScreenState;
use enum_as_inner::EnumAsInner;
use starknet_rust_core::types::Felt;
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreState {
    pub account: AccountState,
    pub toast_queue: VecDeque<String>,
    pub contract_address: String,
    pub chain: String,
    pub version: String,
    pub running: bool,
}

pub enum Effect {
    RequestQuit,
    RequestNavigateTo(ScreenState),
    RequestNavigateBack,
    RequestPopToastAfter(Duration)
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum Intent {
    OnQuit,
    OnOpen(ScreenState),
    OnGoBack,
    OnAccountLoggedIn(LoggedAccount),
    OnShowToast(String),
    OnHideToast
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum AccountState {
    None,
    LoggedIn(LoggedAccount),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoggedAccount {
    pub address: Felt,
    pub kind: AccountKind,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum AccountKind {
    Local,
    Cartridge,
}
