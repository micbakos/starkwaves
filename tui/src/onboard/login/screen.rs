use std::fmt::format;
use crate::app::services::Services;
use crate::app::types::CoreState;
use crate::onboard::login::types::{CliPopupAction, Effect, Intent, LoginOption, State};
use crate::types::nav::NavCommand;
use crate::types::screen::Screen;
use crate::types::{AppEffect, AppIntent};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use std::sync::Arc;
use strum::VariantArray;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::UnboundedSender;
use crate::app::types::Intent::{OnAccountLoggedIn, OnNav, OnShowToast};
use crate::lobby::types::Intent::OnStart;
use crate::types::result::Result;

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
        let mut effects = Vec::<AppEffect>::new();
        match intent {
            Intent::OnPressDown => new_state.press_down(),
            Intent::OnPressUp => new_state.press_up(),
            Intent::OnSelect => {
                if let Some(popup) = new_state.cli_popup {
                    if popup.action == CliPopupAction::Download {
                        panic!("Not implemented yet");
                    }

                    new_state.cli_popup = None;
                } else if state.login_option == LoginOption::Cartridge {
                    if let Some(path) = Services::cartridge_cli_path() {
                        effects.push(Effect::RequestLoginWithCartridge(path).into());
                        new_state.dismiss_popup();
                    } else {
                        new_state.reveal_popup();
                    }
                }
            }
            Intent::OnCliPopupDismiss => new_state.dismiss_popup(),
            Intent::OnCliPopupNextAction => new_state.select_popup_next_action(),
            Intent::OnCliPopupPrevAction => new_state.select_popup_prev_action(),
        }
        (new_state, effects)
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

        if let Some(popup) = state.cli_popup {
            let popup_block = Block::bordered().title("Login with Cartridge");
            let popup_area = area.centered(Constraint::Percentage(60), Constraint::Percentage(20));
            frame.render_widget(Clear, popup_area);
            let inner_area = popup_block.inner(popup_area);
            frame.render_widget(popup_block, popup_area);

            let popup_layout = Layout::default()
                .constraints([Constraint::Fill(1), Constraint::Length(1)])
                .split(inner_area);

            let message = Paragraph::new("Login with Cartridge requires controller cli. Do you want to install controller cli?").centered();
            let [_, message_area, _] = Layout::default()
                .constraints([Constraint::Fill(1), Constraint::Min(1), Constraint::Fill(1)])
                .areas(popup_layout[0]);

            frame.render_widget(message, message_area);

            let buttons_layout =
                Layout::horizontal(CliPopupAction::VARIANTS.iter().map(|_| Constraint::Fill(1)))
                    .split(popup_layout[1]);

            let selected_style = Style::default().reversed();
            let normal_style = Style::default();

            CliPopupAction::VARIANTS
                .iter()
                .enumerate()
                .for_each(|(index, variant)| {
                    let label = match variant {
                        CliPopupAction::Download => "Download",
                        CliPopupAction::Cancel => "Cancel",
                    };

                    let style = if popup.action == *variant {
                        selected_style
                    } else {
                        normal_style
                    };

                    let line = Line::raw(label).centered().style(style);
                    frame.render_widget(line, buttons_layout[index]);
                })
        }
    }

    fn on_key(key: KeyEvent, state: &Self::State) -> Option<Self::Intent> {
        match key.code {
            KeyCode::Up => Some(Intent::OnPressUp),
            KeyCode::Down => Some(Intent::OnPressDown),
            KeyCode::Enter => Some(Intent::OnSelect),
            KeyCode::Right => {
                if state.cli_popup.is_some() {
                    Some(Intent::OnCliPopupNextAction)
                } else {
                    None
                }
            }
            KeyCode::Left => {
                if state.cli_popup.is_some() {
                    Some(Intent::OnCliPopupPrevAction)
                } else {
                    None
                }
            }
            KeyCode::Esc => {
                if state.cli_popup.is_some() {
                    Some(Intent::OnCliPopupDismiss)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    async fn run(
        effect: Self::Effect,
        services: Arc<Services>,
        intents: UnboundedSender<AppIntent>,
    ) -> std::result::Result<(), SendError<AppIntent>> {
        match effect {
            Effect::RequestLoginWithCartridge(cli_path) => {
                let logged_account_result = services.resolve_cartridge_account(cli_path)
                    .await;

                match logged_account_result {
                    Ok(logged_account) => {
                        intents.send(OnAccountLoggedIn(logged_account.clone()).into())?;
                        intents.send(Intent::OnCliPopupDismiss.into())?;

                        let lobby_state = crate::lobby::types::State::new(logged_account);
                        intents.send(OnNav(NavCommand::ResetTo(lobby_state.into())).into())?;
                        intents.send(OnStart.into())?;
                    }
                    Err(error) => {
                        intents.send(OnShowToast(format!("{}", error)).into())?;
                    }
                }
            }
        }

        Ok(())
    }
}
