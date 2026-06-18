use enum_as_inner::EnumAsInner;

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum ScreenState {
    Start(crate::onboard::start::screen::State)
}