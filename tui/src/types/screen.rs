use crate::app::types::CoreState;
use crate::types::AppEffect;
use crate::types::AppIntent;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::fmt::Debug;
use tokio::sync::mpsc::UnboundedSender;

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

    fn on_key(key: KeyEvent) -> Option<Self::Intent>;

    async fn run(effect: Self::Effect, intents: UnboundedSender<AppIntent>);
}
