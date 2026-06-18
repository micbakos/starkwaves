use crossterm::event::KeyEvent;
use enum_as_inner::EnumAsInner;
use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::text::Text;
use ratatui::widgets::GraphType::Line;
use ratatui::widgets::Paragraph;
use crate::app::core::{AccountState, CoreState};
use crate::types::screen::Screen;
use crate::types::state::ScreenState;

pub struct AppScreen;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub core: CoreState,
    pub screens: Vec<ScreenState>
}

impl State {
    pub fn start() -> Self {
        Self {
            core: CoreState {
                account: AccountState::None,
                toast: None,
                running: true,
            },
            screens: vec![ScreenState::Start(
                crate::onboard::start::screen::State::new()
            )]
        }
    }
}

pub enum Effect {
}

#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum Intent {
    Quit,
}

pub fn reduce(state: &State, intent: Intent) -> (State, Vec<Effect>) {
    let mut new_state = state.clone();
    match intent {
        Intent::Quit => {
            new_state.core.running = false;
        }
    }

    (new_state, vec![])
}

impl Screen for AppScreen {
    type Intent = Intent;
    type Effect = Effect;
    type State = State;

    fn reduce(state: &Self::State, intent: Self::Intent, core: &CoreState) -> (Self::State, Vec<Self::Effect>) {
        todo!()
    }

    fn render(state: &Self::State, core: &CoreState, frame: &mut Frame, area: Rect) {
        let version_text = Text::raw("v1.0.0").right_aligned();
        frame.render_widget(
            version_text,
            area
        )
    }

    fn on_key(key: KeyEvent) -> Option<Self::Intent> {
        None
    }

    async fn run(effect: Self::Effect) -> Self::Intent {
        todo!()
    }
}