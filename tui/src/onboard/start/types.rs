use crate::types::menu_iterable::MenuIterable;
use strum::VariantArray;

#[derive(Copy, Debug, Clone, PartialEq, Eq, VariantArray)]
pub enum Menu {
    Start,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub selected_button: Menu,
}

impl State {
    pub fn new() -> Self {
        Self {
            selected_button: Menu::first(),
        }
    }

    pub fn press_down(&mut self) {
        self.selected_button = self.selected_button.next();
    }

    pub fn press_up(&mut self) {
        self.selected_button = self.selected_button.prev();
    }
}

pub enum Intent {
    OnPressDown,
    OnPressUp,
    OnSelect,
}

pub enum Effect {}
