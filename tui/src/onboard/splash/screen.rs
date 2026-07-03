use crate::app::services::Services;
use crate::app::types::{CoreState, LoggedAccount};
use crate::onboard::splash::types::{Effect, Intent, State};
use crate::types::result::Result;
use crate::types::screen::Screen;
use crate::types::{AppEffect, AppIntent};
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::Paragraph;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use crate::app::types::Intent::{OnAccountLoggedIn, OnShowToast};

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
    ) -> Result<()> {
        match effect {
            Effect::RequestResolveStoredSession => {
                let session = services.resolve_session().await?;

                if let Some(session) = session {
                    let logged_account: LoggedAccount = session.account.into();
                    intents.send(OnAccountLoggedIn(logged_account.clone()).into())?;

                    intents.send(
                        OnShowToast(format!("Logged in with {} ({:?})", logged_account.address, logged_account.kind)).into()
                    )?;
                } else {
                    let screen = crate::onboard::start::types::State::new();
                    intents.send(crate::app::types::Intent::OnOpen(screen.into()).into())?;
                }
            }
        }

        Ok(())
    }
}
