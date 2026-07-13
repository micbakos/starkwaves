use crate::app::services::Services;
use crate::app::types::{CoreState, LoggedAccount};
use crate::onboard::splash::types::{Effect, Intent, State};
use crate::types::nav::NavCommand;
use crate::types::screen::Screen;
use crate::types::{AppEffect, AppIntent};
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::Paragraph;
use std::sync::Arc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::UnboundedSender;
use crate::app::types::Intent::{OnAccountLoggedIn, OnNav, OnShowToast};
use crate::lobby::screen::LobbyScreen;
use crate::lobby::types::Intent::OnStart;

pub struct SplashScreen {}

impl Screen for SplashScreen {
    type Intent = Intent;
    type Effect = Effect;
    type State = State;

    fn reduce(
        state: &Self::State,
        intent: Self::Intent,
        _core: &CoreState,
    ) -> (Self::State, Vec<AppEffect>) {
        let mut new_state = state.clone();
        let mut effects = vec![];

        match intent {
            Intent::OnStart => {
                effects.push(Effect::RequestResolveStoredSession.into());
            }
        }

        (new_state, effects)
    }

    fn render(_state: &Self::State, _core: &CoreState, frame: &mut Frame, area: Rect) {
        let [logo_area] = Layout::vertical([Constraint::Length(1)])
            .flex(Flex::Center)
            .areas(area);

        let logo = Paragraph::new("Starkwaves").centered().bold();
        frame.render_widget(logo, logo_area);
    }

    fn on_key(_key: KeyEvent, _state: &Self::State) -> Option<Self::Intent> {
        None
    }

    async fn run(
        effect: Self::Effect,
        services: Arc<Services>,
        intents: UnboundedSender<AppIntent>,
    ) -> Result<(), SendError<AppIntent>> {
        match effect {
            Effect::RequestResolveStoredSession => {
                let result = services.resolve_session().await;

                if let Ok(Some(account)) = result {
                    intents.send(OnAccountLoggedIn(account.clone()).into())?;

                    let lobby_state = crate::lobby::types::State::new(account);
                    intents.send(OnNav(NavCommand::Replace(lobby_state.into())).into())?;
                    intents.send(OnStart.into())?;
                } else if let Err(err) = result {
                    intents.send(OnShowToast(err.to_string()).into())?;
                } else {
                    let screen = crate::onboard::start::types::State::new();
                    intents.send(OnNav(NavCommand::Replace(screen.into())).into())?;
                }
            }
        }

        Ok(())
    }
}
