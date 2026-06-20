use derive_more::From;
use crate::{app, onboard};

#[derive(From)]
pub enum Intent {
    App(app::screen::Intent),
    Start(onboard::start::screen::Intent),
}