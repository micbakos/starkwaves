use crate::app;

pub enum Effect {
    App(app::screen::Effect),
    Start(crate::onboard::start::screen::Effect),
}