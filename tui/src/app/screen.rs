use crate::app::types::{AppState, CoreState, Effect, Intent};
use crate::onboard::{login, start};
use crate::types::screen::Screen;
use crate::types::{AppEffect, AppIntent, ScreenState};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Constraint::{Length, Percentage};
use ratatui::layout::{Direction, Layout, Rect};
use ratatui::text::Text;
use tokio::sync::mpsc::UnboundedSender;

pub struct AppScreen;

impl Screen for AppScreen {
    type Intent = AppIntent;
    type Effect = AppEffect;
    type State = AppState;

    fn reduce(
        state: &Self::State,
        intent: Self::Intent,
        _core: &CoreState,
    ) -> (Self::State, Vec<AppEffect>) {
        match intent {
            AppIntent::App(intent) => {
                let mut state = state.clone();

                match intent {
                    Intent::Quit => {
                        state.core.running = false;
                    }
                    Intent::Open(screen_state) => {
                        state.screens.insert(0, screen_state);
                    }
                    Intent::GoBack => {
                        if state.screens.len() > 1 {
                            state.screens.remove(0);
                        } else {
                            state.core.running = false;
                        }
                    }
                }

                (state, vec![])
            }

            // TODO: Boilerplate reducers
            AppIntent::Start(intent) => {
                let mut state = state.clone();
                let screen = state
                    .screens
                    .first_mut()
                    .expect("Received intent but no screen exists in stack.");
                let start_state = screen
                    .as_start_mut()
                    .expect("Received start intent but no start screen.");

                let (screen_state, effects) =
                    start::screen::StartScreen::reduce(start_state, intent, &state.core);

                state.screens[0] = ScreenState::Start(screen_state);

                (state, effects)
            }
            AppIntent::Login(intent) => {
                let mut state = state.clone();
                let screen = state
                    .screens
                    .first_mut()
                    .expect("Received intent but no screen exists in stack.");
                let login_state = screen
                    .as_login_mut()
                    .expect("Received login intent but no login screen.");

                let (screen_state, effects) =
                    login::screen::LoginScreen::reduce(login_state, intent, &state.core);

                state.screens[0] = ScreenState::Login(screen_state);

                (state, effects)
            }
        }
    }

    fn render(state: &Self::State, _core: &CoreState, frame: &mut Frame, _area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Percentage(100), Length(1)])
            .split(frame.area());

        let screen = state.screens.first().expect("No screen exists to render");

        // TODO: boilerplate renderers
        match screen {
            ScreenState::Start(screen_state) => {
                start::screen::StartScreen::render(screen_state, &state.core, frame, layout[0])
            }
            ScreenState::Login(screen_state) => {
                login::screen::LoginScreen::render(screen_state, &state.core, frame, layout[0])
            }
        }

        let version_text = Text::raw("v1.0.0").right_aligned();
        frame.render_widget(version_text, layout[1])
    }

    fn on_key(key: KeyEvent) -> Option<Self::Intent> {
        match key.code {
            KeyCode::Esc => Some(Intent::GoBack.into()),
            _ => None,
        }
    }

    async fn run(effect: Self::Effect, intents: UnboundedSender<AppIntent>) {
        match effect {
            AppEffect::App(effect) => {
                // TODO Error?
                match effect {
                    Effect::RequestQuit => {
                        intents.send(Intent::Quit.into());
                    }
                    Effect::RequestNavigateTo(screen_state) => {
                        intents.send(Intent::Open(screen_state).into());
                    }
                    Effect::RequestNavigateBack => {
                        intents.send(Intent::GoBack.into());
                    }
                }
            }

            // TODO: boilerplate effects
            AppEffect::Start(effect) => {
                start::screen::StartScreen::run(effect, intents).await;
            }
            AppEffect::Login(effect) => {
                login::screen::LoginScreen::run(effect, intents).await;
            }
        }
    }
}
