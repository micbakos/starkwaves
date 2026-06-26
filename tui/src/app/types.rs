use crate::app::services::OnChainData;
use crate::onboard::{splash, start};
use crate::types::ScreenState;
use enum_as_inner::EnumAsInner;
use starknet_rust_core::chain_id;
use starknet_rust_core::types::Felt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub core: CoreState,
    pub screens: Vec<ScreenState>,
}

impl AppState {
    pub fn start(
        on_chain_data: &OnChainData,
    ) -> Self {
        let contract_address_string = on_chain_data.contract_address.to_fixed_hex_string();
        let chain = if on_chain_data.chain_id == chain_id::MAINNET {
            "Mainnet"
        } else {
            "Sepolia"
        }.to_string();

        Self {
            core: CoreState {
                account: AccountState::None,
                toast: None,
                contract_address: contract_address_string,
                chain,
                version: env!("CARGO_PKG_VERSION").to_string(),
                running: true,
            },
            screens: vec![splash::types::State::new().into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreState {
    pub account: AccountState,
    pub toast: Option<String>,
    pub contract_address: String,
    pub chain: String,
    pub version: String,
    pub running: bool,
}

pub enum Effect {
    RequestQuit,
    RequestNavigateTo(ScreenState),
    RequestNavigateBack,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum Intent {
    OnQuit,
    OnOpen(ScreenState),
    OnGoBack,
    OnAccountLoggedIn(LoggedAccount),
    OnShowToast(String),
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
