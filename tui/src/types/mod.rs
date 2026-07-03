use std::collections::VecDeque;
use crate::onboard::login::screen::LoginScreen;
use crate::onboard::splash::screen::SplashScreen;
use crate::onboard::start::screen::StartScreen;
use crate::screens;
use derive_more::From;
use enum_as_inner::EnumAsInner;
use starknet_rust_core::chain_id;
use crate::app::services::OnChainData;
use crate::app::types::{AccountState, CoreState, ToastsState};
use crate::onboard::splash;

pub(crate) mod menu_iterable;
pub(crate) mod screen;
pub(crate) mod screens_macro;
pub(crate) mod error;
pub(crate) mod result;

screens!(
    Splash => SplashScreen,
    Start => StartScreen,
    Login => LoginScreen
);

#[derive(From)]
pub enum AppEffect {
    App(crate::app::types::Effect),
    Screen(ScreenEffect),
}

#[derive(From)]
pub enum AppIntent {
    App(crate::app::types::Intent),
    Screen(ScreenIntent),
}

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
                toasts: ToastsState {
                    queue: Default::default(),
                    current: None,
                },
                contract_address: contract_address_string,
                chain,
                version: env!("CARGO_PKG_VERSION").to_string(),
                running: true,
            },
            screens: vec![splash::types::State::new().into()],
        }
    }
}