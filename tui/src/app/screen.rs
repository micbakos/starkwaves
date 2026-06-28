use crate::app::services::Services;
use crate::app::types::{AccountState, CoreState, Effect, Intent};
use crate::onboard::{login, splash, start};
use crate::types::screen::Screen;
use crate::types::{screens_reduce, screens_render, AppEffect, AppIntent, AppState, ScreenEffect, ScreenState, screens_run};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use std::sync::Arc;
use crate::types::result::Result;
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
                    Intent::OnQuit => {
                        state.core.running = false;
                    }
                    Intent::OnOpen(screen_state) => {
                        state.screens.insert(0, screen_state);
                    }
                    Intent::OnGoBack => {
                        if state.screens.len() > 1 {
                            state.screens.remove(0);
                        } else {
                            state.core.running = false;
                        }
                    }
                    Intent::OnAccountLoggedIn(logged_account) => {
                        state.core.account = AccountState::LoggedIn(logged_account);
                    }
                    Intent::OnShowToast(message) => {
                        state.core.toast = Some(message.clone());
                    }
                }

                (state, vec![])
            }

            AppIntent::Screen(screen_intent) => {
                let mut state = state.clone();

                let top_screen_state = state
                    .screens
                    .first()
                    .expect("Received intent but no screen exists in stack.");

                let (new_screen_state, effects) = screens_reduce(
                    screen_intent,
                    top_screen_state.clone(),
                    &state.core
                );

                state.screens[0] = new_screen_state;

                (state, effects)
            }
        }
    }

    fn render(state: &Self::State, core: &CoreState, frame: &mut Frame, _area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Length(1)])
            .split(frame.area());

        let screen = state.screens.first().expect("No screen exists to render");

        screens_render(&screen, &state.core, frame, layout[0]);
        render_core_details(frame, layout[1], core);
    }

    fn on_key(key: KeyEvent, _state: &Self::State) -> Option<Self::Intent> {
        match key.code {
            KeyCode::Esc => Some(Intent::OnGoBack.into()),
            _ => None,
        }
    }

    async fn run(effect: Self::Effect, services: Arc<Services>, intents: UnboundedSender<AppIntent>) -> Result<()> {
        match effect {
            AppEffect::App(effect) => {
                match effect {
                    Effect::RequestQuit => {
                        intents.send(Intent::OnQuit.into())?;
                    }
                    Effect::RequestNavigateTo(screen_state) => {
                        intents.send(Intent::OnOpen(screen_state).into())?;
                    }
                    Effect::RequestNavigateBack => {
                        intents.send(Intent::OnGoBack.into())?;
                    }
                }
            }
            AppEffect::Screen(screen_effect) => screens_run(screen_effect, services, intents).await?,
        }

        Ok(())
    }
}

fn render_core_details(frame: &mut Frame, area: Rect, core: &CoreState) {
    let [details_area, version_area] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Min(1),
    ]).areas(area);

    let [contract_area, _, chain_id_area] = Layout::horizontal([
        Constraint::Min(1),
        Constraint::Length(2),
        Constraint::Min(1),
    ]).areas(details_area);

    let chain_id_text = Text::raw(core.chain.as_str());
    frame.render_widget(chain_id_text, chain_id_area);

    let version_text = Line::from(vec![
        Span::from("Game: "),
        Span::styled(core.contract_address.as_str(), Style::default().bold()),
    ]);
    frame.render_widget(version_text, contract_area);

    let version_text = Text::raw(format!("v{}", core.version)).right_aligned();
    frame.render_widget(version_text, version_area);
}