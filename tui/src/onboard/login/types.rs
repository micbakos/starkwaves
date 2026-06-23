use crate::types::menu_iterable::MenuIterable;
use strum::VariantArray;

#[derive(Copy, Debug, Clone, PartialEq, Eq, VariantArray)]
pub enum LoginOption {
    Local,
    Cartridge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub login_option: LoginOption,
}

impl State {
    pub fn new() -> Self {
        Self {
            login_option: LoginOption::first(),
        }
    }

    pub fn press_down(&mut self) {
        self.login_option = self.login_option.next();
    }

    pub fn press_up(&mut self) {
        self.login_option = self.login_option.prev();
    }
}

pub enum Intent {
    OnPressDown,
    OnPressUp,
    OnSelect,
}

pub enum Effect {}
