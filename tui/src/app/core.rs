use enum_as_inner::EnumAsInner;
use starknet_rust_core::types::Felt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreState {
    pub account: AccountState,
    pub toast: Option<String>,
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum AccountState {
    None,
    LoggedIn { address: Felt, kind: AccountKind }
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum AccountKind {
    Local,
    Cartridge
}