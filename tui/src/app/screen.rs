use crate::app::services::Services;
use crate::app::types::{AccountState, CoreState, Effect, Intent};
use crate::types::result::Result;
use crate::types::screen::Screen;
use crate::types::{
    AppEffect, AppIntent, AppState, screens_reduce, screens_render,
    screens_run,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::sleep;

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
                let mut effects = vec![];

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
                        if state.core.toasts.current.is_none() {
                            state.core.toasts.current = Some(message.clone());
                            effects.push(Effect::RequestPopToastAfter(Duration::from_secs(2)).into())
                        } else {
                            state.core.toasts.queue.push_back(message);
                        }
                    },
                    Intent::OnHideToast => {
                        let queued = state.core.toasts.queue.front();

                        if let Some(message) = queued {
                            state.core.toasts.current = Some(message.clone());
                            effects.push(Effect::RequestPopToastAfter(Duration::from_secs(2)).into())
                        } else {
                            state.core.toasts.current = None;
                        }
                    }
                }

                (state, effects)
            }

            AppIntent::Screen(screen_intent) => {
                let mut state = state.clone();

                let top_screen_state = state
                    .screens
                    .first()
                    .expect("Received intent but no screen exists in stack.");

                let (new_screen_state, effects) =
                    screens_reduce(screen_intent, top_screen_state.clone(), &state.core);

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

        if let Some(toast) = &state.core.toasts.current {
            toast_render(frame, layout[0], toast.clone());
        }

        render_core_details(frame, layout[1], core);
    }

    fn on_key(key: KeyEvent, _state: &Self::State) -> Option<Self::Intent> {
        match key.code {
            KeyCode::Esc => Some(Intent::OnGoBack.into()),
            _ => None,
        }
    }

    async fn run(
        effect: Self::Effect,
        services: Arc<Services>,
        intents: UnboundedSender<AppIntent>,
    ) -> Result<()> {
        match effect {
            AppEffect::App(effect) => match effect {
                Effect::RequestQuit => {
                    intents.send(Intent::OnQuit.into())?;
                }
                Effect::RequestNavigateTo(screen_state) => {
                    intents.send(Intent::OnOpen(screen_state).into())?;
                }
                Effect::RequestNavigateBack => {
                    intents.send(Intent::OnGoBack.into())?;
                }
                Effect::RequestPopToastAfter(duration) => {
                    sleep(duration).await;
                    intents.send(Intent::OnHideToast.into())?;
                }
            },
            AppEffect::Screen(screen_effect) => {
                screens_run(screen_effect, services, intents).await?
            }
        }

        Ok(())
    }
}

fn render_core_details(frame: &mut Frame, area: Rect, core: &CoreState) {
    let [details_area, version_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Min(1)]).areas(area);

    let address_text = Paragraph::new(
        Line::from(vec![
            Span::from("Game: "),
            Span::styled(core.contract_address.as_str(), Style::default().bold()),
        ])
    );
    let [address_area, _, chain_id_area] = Layout::horizontal([
        Constraint::Length(address_text.line_width() as u16),
        Constraint::Length(2),
        Constraint::Length(core.chain.len() as u16),
    ])
    .areas(details_area);

    let chain_id_text = Text::raw(core.chain.as_str());
    frame.render_widget(chain_id_text, chain_id_area);

    frame.render_widget(address_text, address_area);

    let version_text = Text::raw(format!("v{}", core.version)).right_aligned();
    frame.render_widget(version_text, version_area);
}

fn toast_render(frame: &mut Frame, area: Rect, toast: String) {
    let message_area = (area.width.saturating_sub(4) * 3 / 4).max(1);

    let text = Text::from(toast);

    let message = Paragraph::new(text).wrap(Wrap { trim: true });
    let inner_height = message.line_count(message_area) as u16;

    let popup_width = message_area + 4;
    let popup_height = inner_height + 2;

    let [_, bottom] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(popup_height)
    ]).areas(area);

    let [popup_area] = Layout::horizontal([Constraint::Length(popup_width)])
        .flex(Flex::Center)
        .areas(bottom);

    let popup_block = Block::bordered().title("Message");
    let inner_area = popup_block.inner(popup_area);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup_block, popup_area);
    frame.render_widget(message, inner_area);
}
