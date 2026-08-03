use crate::app::services::Services;
use crate::app::types::AccountState;
use crate::app::types::Effect::RequestNav;
use crate::onboard::start::types::{Effect, Intent, Menu, State};
use crate::types::AppEffect;
use crate::types::AppIntent;
use crate::types::nav::NavCommand;
use crate::types::screen::Screen;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use std::sync::Arc;
use strum::VariantArray;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::error::SendError;

pub struct StartScreen;

impl Screen for StartScreen {
    type Intent = Intent;
    type Effect = Effect;
    type State = State;

    fn reduce(
        state: &Self::State,
        intent: Self::Intent,
        core: &crate::app::types::CoreState,
    ) -> (Self::State, Vec<AppEffect>) {
        let mut new_state = state.clone();
        let mut effects: Vec<AppEffect> = vec![];
        match intent {
            Intent::OnPressDown => new_state.press_down(),
            Intent::OnPressUp => new_state.press_up(),
            Intent::OnSelect => match state.selected_button {
                Menu::Start => {
                    if let AccountState::LoggedIn(logged_account) = &core.account {
                        let lobby_screen_state =
                            crate::lobby::types::State::new(logged_account.clone());
                        effects
                            .push(RequestNav(NavCommand::ResetTo(lobby_screen_state.into())).into())
                    } else {
                        let login_screen_state = crate::login::types::State::new();
                        effects.push(RequestNav(NavCommand::Push(login_screen_state.into())).into())
                    }
                }
                Menu::Quit => effects.push(crate::app::types::Effect::RequestQuit.into()),
            },
        };

        (new_state, effects)
    }

    fn render(
        state: &Self::State,
        _core: &crate::app::types::CoreState,
        frame: &mut Frame,
        area: Rect,
    ) {
        let block = Block::default()
            .border_style(Style::default().fg(Color::Magenta))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let lines = Menu::VARIANTS
            .into_iter()
            .map(|menu| {
                let label = match menu {
                    Menu::Start => "Start Game",
                    Menu::Quit => "Quit",
                };

                let line_style = if state.selected_button == *menu {
                    Style::default().reversed()
                } else {
                    Style::default()
                };

                Line::raw(label).style(line_style).centered()
            })
            .collect::<Vec<_>>();

        let [buttons_area] = Layout::vertical([Constraint::Length(lines.len() as u16)])
            .flex(Flex::Center)
            .areas(inner_area);

        frame.render_widget(Paragraph::new(lines), buttons_area);
    }

    fn on_key(key: KeyEvent, _state: &Self::State) -> Option<Self::Intent> {
        match key.code {
            KeyCode::Up => Some(Intent::OnPressUp),
            KeyCode::Down => Some(Intent::OnPressDown),
            KeyCode::Enter => Some(Intent::OnSelect),
            _ => None,
        }
    }

    async fn run(
        _effect: Self::Effect,
        services: Arc<Services>,
        _intents: UnboundedSender<AppIntent>,
    ) -> std::result::Result<(), SendError<AppIntent>> {
        Ok(())
    }
}
