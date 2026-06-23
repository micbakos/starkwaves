use crate::app::types::CoreState;
use crate::onboard::login::types::{Effect, Intent, LoginOption, State};
use crate::types::screen::Screen;
use crate::types::{AppEffect, AppIntent};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::{Color, Line, Style};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use strum::VariantArray;
use tokio::sync::mpsc::UnboundedSender;

pub struct LoginScreen {}

impl Screen for LoginScreen {
    type Intent = Intent;
    type Effect = Effect;
    type State = State;

    fn reduce(
        state: &Self::State,
        intent: Self::Intent,
        _core: &CoreState,
    ) -> (Self::State, Vec<AppEffect>) {
        let mut new_state = state.clone();
        match intent {
            Intent::OnPressDown => new_state.press_down(),
            Intent::OnPressUp => new_state.press_up(),
            Intent::OnSelect => {}
        }
        (new_state, vec![])
    }

    fn render(state: &Self::State, _core: &CoreState, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .border_style(Style::default().fg(Color::Magenta))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let lines = LoginOption::VARIANTS
            .into_iter()
            .map(|option| {
                let label = match option {
                    LoginOption::Local => "Local Account",
                    LoginOption::Cartridge => "Cartridge",
                };

                let line_style = if state.login_option == *option {
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

    fn on_key(key: KeyEvent) -> Option<Self::Intent> {
        match key.code {
            KeyCode::Up => Some(Intent::OnPressUp),
            KeyCode::Down => Some(Intent::OnPressDown),
            KeyCode::Enter => Some(Intent::OnSelect),
            _ => None,
        }
    }

    async fn run(_effect: Self::Effect, _intents: UnboundedSender<AppIntent>) {}
}
