use derive_more::From;
use enum_as_inner::EnumAsInner;

pub(crate) mod menu_iterable;
pub(crate) mod screen;

#[derive(From)]
pub enum AppEffect {
    App(crate::app::types::Effect),
    Start(crate::onboard::start::types::Effect),
    Login(crate::onboard::login::types::Effect),
}

#[derive(From)]
pub enum AppIntent {
    App(crate::app::types::Intent),
    Start(crate::onboard::start::types::Intent),
    Login(crate::onboard::login::types::Intent),
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner, From)]
pub enum ScreenState {
    Start(crate::onboard::start::types::State),
    Login(crate::onboard::login::types::State),
}
