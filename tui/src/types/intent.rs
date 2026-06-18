use crate::{app, onboard};

pub enum Intent {
    App(app::screen::Intent),
    Start(onboard::start::screen::Intent),
}