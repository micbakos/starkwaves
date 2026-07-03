
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub is_loading: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            is_loading: true,
        }
    }
}

pub enum Intent {
    OnStart
}

pub enum Effect {
    RequestResolveStoredSession
}