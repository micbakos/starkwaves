use crate::app::services::Services;
use crate::app::types::CoreState;
use crate::app::types::Intent::{OnAccountLoggedIn, OnNav, OnShowToast};
use crate::onboard::login::types::{CliPopupAction, Effect, Intent, LoginOption, State};
use crate::popup::render_popup;
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
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::error::SendError;

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
                } else {
                    match state.login_option {
                        #[cfg(debug_assertions)]
                        LoginOption::Local => {
                            effects.push(Effect::RequestLoginWithPrivateKeyFromEnv.into())
                        }
                        LoginOption::Cartridge => {
                            if let Some(path) = Services::cartridge_cli_path() {
                                effects.push(Effect::RequestLoginWithCartridge(path).into());
                                new_state.dismiss_popup();
                            } else {
                                new_state.reveal_popup();
                            }
                        }
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
            .iter()
            .map(|option| {
                let label = match option {
                    #[cfg(debug_assertions)]
                    LoginOption::Local => "Local Account (Dev)",
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
            render_popup(
                frame,
                area,
                Some("Login with Cartridge"),
                "Login with Cartridge requires controller cli. Do you want to install controller cli?",
                &popup.action,
                CliPopupAction::VARIANTS,
            );
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
                let logged_account_result = services.resolve_cartridge_account(cli_path).await;

                match logged_account_result {
                    Ok(logged_account) => {
                        intents.send(OnAccountLoggedIn(logged_account.clone()).into())?;
                        intents.send(Intent::OnCliPopupDismiss.into())?;

                        let lobby_state = crate::lobby::types::State::new(logged_account);
                        intents.send(OnNav(NavCommand::ResetTo(lobby_state.into())).into())?;
                    }
                    Err(error) => {
                        intents.send(OnShowToast(format!("{}", error)).into())?;
                    }
                }
            }
            #[cfg(debug_assertions)]
            Effect::RequestLoginWithPrivateKeyFromEnv => {
                let env_account_result = services.resolve_local_account_from_env().await;

                match env_account_result {
                    Ok(logged_account) => {
                        intents.send(OnAccountLoggedIn(logged_account.clone()).into())?;

                        let lobby_state = crate::lobby::types::State::new(logged_account);
                        intents.send(OnNav(NavCommand::ResetTo(lobby_state.into())).into())?;
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
