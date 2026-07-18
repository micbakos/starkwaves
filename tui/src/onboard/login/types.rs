use std::path::PathBuf;
use crate::types::menu_iterable::MenuIterable;
use strum::VariantArray;
use starkwaves_client::types::account::cartridge_account::CartridgeAccount;
use starkwaves_client::types::result::Result;
use crate::app::types::LoggedAccount;

#[derive(Copy, Debug, Clone, PartialEq, Eq, VariantArray)]
pub enum LoginOption {
    #[cfg(debug_assertions)]
    Local,
    Cartridge,
}

#[derive(Copy, Debug, Clone, PartialEq, Eq, VariantArray)]
pub enum CliPopupAction {
    Download,
    Cancel
}

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub struct CliPopup {
    pub action: CliPopupAction
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub login_option: LoginOption,
    pub cli_popup: Option<CliPopup>,
}

impl State {
    pub fn new() -> Self {
        Self {
            login_option: LoginOption::first(),
            cli_popup: None,
        }
    }

    pub fn press_down(&mut self) {
        self.login_option = self.login_option.next();
    }

    pub fn press_up(&mut self) {
        self.login_option = self.login_option.prev();
    }

    pub fn reveal_popup(&mut self) {
        self.cli_popup = Some(CliPopup {
            action: CliPopupAction::first(),
        })
    }

    pub fn dismiss_popup(&mut self) {
        self.cli_popup = None;
    }

    pub fn select_popup_next_action(&mut self) {
        if let Some(popup) = &mut self.cli_popup {
            popup.action = popup.action.next();
        }
    }

    pub fn select_popup_prev_action(&mut self) {
        if let Some(popup) = &mut self.cli_popup {
            popup.action = popup.action.prev();
        }
    }
}

pub enum Intent {
    OnPressDown,
    OnPressUp,
    OnSelect,
    OnCliPopupDismiss,
    OnCliPopupNextAction,
    OnCliPopupPrevAction,
}

pub enum Effect {
    RequestLoginWithCartridge(PathBuf),
    #[cfg(debug_assertions)]
    RequestLoginWithPrivateKeyFromEnv
}
