use crate::app::services::Services;
use crate::app::types::CoreState;
use crate::onboard::splash::types::{Effect, Intent, State};
use crate::types::screen::Screen;
use crate::types::{AppEffect, AppIntent};
use crossterm::event::KeyEvent;
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::Paragraph;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use crate::types::result::Result;

pub struct SplashScreen {}

impl Screen for SplashScreen {
    type Intent = Intent;
    type Effect = Effect;
    type State = State;

    fn reduce(state: &Self::State, intent: Self::Intent, core: &CoreState) -> (Self::State, Vec<AppEffect>) {
        let mut new_state = state.clone();
        let mut effects = vec![];

        match intent {
            Intent::OnStart => {
                effects.push(Effect::RequestCheckLoggedAccount.into());
            }
        }

        (new_state, effects)
    }

    fn render(state: &Self::State, core: &CoreState, frame: &mut Frame, area: Rect) {
        let [_, logo_area, _] = Layout::default()
            .constraints([Constraint::Fill(1), Constraint::Min(1), Constraint::Fill(1)])
            .areas(area);

        let logo = Paragraph::new("Starkwaves").centered().bold();
        frame.render_widget(logo, logo_area);
    }

    fn on_key(key: KeyEvent, state: &Self::State) -> Option<Self::Intent> {
        None
    }

    async fn run(effect: Self::Effect, services: Arc<Services>, intents: UnboundedSender<AppIntent>) -> Result<()> {
        match effect {
            Effect::RequestCheckLoggedAccount => {
                // TODO check for active session
            }
        }

        Ok(())
    }
}