use derive_more::From;
use crate::app;

#[derive(From)]
pub enum Effect {
    App(app::screen::Effect),
    Start(crate::onboard::start::screen::Effect),
}