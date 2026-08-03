use crate::lobby::types::LobbyVariant;
use crate::types::nav::{NavCommand, SettleNavCommand};
use crate::types::{ScreenIntent, ScreenState};
use enum_as_inner::EnumAsInner;
use starknet_rust_core::types::Felt;
use starkwaves_client::game::game::GameUpdate;
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreState {
    pub account: AccountState,
    pub toasts: ToastsState,
    pub contract_address: String,
    pub chain: String,
    pub version: String,
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastsState {
    pub queue: VecDeque<String>,
    pub current: Option<String>,
}

pub enum Effect {
    RequestQuit,
    RequestNav(NavCommand),
    RequestSettleNav(SettleNavCommand),
    RequestPopToastAfter(Duration),
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner, derive_more::From)]
pub enum Intent {
    OnQuit,
    OnNav(NavCommand),
    OnSettleNav(SettleNavCommand),
    OnAccountLoggedIn(LoggedAccount),
    OnAccountLoggedOut,
    OnGameUpdate(GameUpdate),
    OnShowToast(String),
    OnHideToast,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum AccountState {
    None,
    LoggedIn(LoggedAccount),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoggedAccount {
    pub address: Felt,
    pub username: String,
    pub kind: AccountKind,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum AccountKind {
    Local,
    Cartridge,
}
