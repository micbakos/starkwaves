use crate::app::services::Services;
use crate::app::types::CoreState;
use crate::types::AppEffect;
use crate::types::AppIntent;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::error::SendError;

pub trait Screen {
    type Intent;
    type Effect;

    type State: Debug + Clone + PartialEq + Eq;

    fn reduce(
        state: &Self::State,
        intent: Self::Intent,
        core: &CoreState,
    ) -> (Self::State, Vec<AppEffect>);

    fn render(state: &Self::State, core: &CoreState, frame: &mut Frame, area: Rect);

    fn on_key(key: KeyEvent, state: &Self::State) -> Option<Self::Intent>;

    fn on_push_effect() -> Option<Self::Effect> {
        None
    }

    fn on_pop_effect() -> Option<Self::Effect> {
        None
    }

    async fn run(
        effect: Self::Effect,
        services: Arc<Services>,
        intents: UnboundedSender<AppIntent>,
    ) -> Result<(), SendError<AppIntent>>;
}
