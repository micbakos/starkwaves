use crate::app::core::CoreState;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::Frame;
use std::fmt::Debug;
use tokio::sync::mpsc::UnboundedSender;
use crate::types::effect::Effect;
use crate::types::intent::Intent;

pub trait Screen {
    type Intent;
    type Effect;

    type State: Debug + Clone + PartialEq + Eq;

    fn reduce(state: &Self::State, intent: Self::Intent, core: &CoreState) -> (Self::State, Vec<Effect>);

    fn render(state: &Self::State, core: &CoreState, frame: &mut Frame, area: Rect);

    fn on_key(key: KeyEvent) -> Option<Self::Intent>;

    async fn run(effect: Self::Effect, intents: UnboundedSender<Intent>);
}