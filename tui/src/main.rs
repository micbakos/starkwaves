mod app;
mod onboard;
mod types;

use crate::app::screen::AppScreen;
use crate::app::services::{OnChainData, Services};
use crate::app::types::AppState;
use crate::onboard::{splash, start};
use crate::types::screen::Screen;
use color_eyre::Result;
use crossterm::event::Event;
use dotenv::dotenv;
use onboard::login;
use ratatui::DefaultTerminal;
use starknet_rust_core::types::Felt;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use starknet_rust_core::chain_id;
use starknet_rust_core::utils::cairo_short_string_to_felt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{mpsc, watch};
use url::Url;
use types::AppEffect;
use types::AppIntent;
use types::ScreenState;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let preset = env::var("PRESET").unwrap_or_else(|_| "Should have PRESET in .env".to_string());
    dotenv::from_filename(format!(".env.{}", preset)).ok();

    let contract_address = env::var("CONTRACT_ADDR")
        .map(|a| Felt::from_hex(a.as_str()).expect("Invalid CONTRACT_ADDR"))
        .expect("CONTRACT_ADDR is not set");
    let chain_id = env::var("CHAIN_ID")
        .map(|a| cairo_short_string_to_felt(a.as_str()).expect("Invalid CHAIN_ID"))
        .unwrap_or(chain_id::MAINNET);
    let rpc_url = env::var("RPC_URL")
        .map(|a| Url::from_str(a.as_str()).expect("Invalid RPC_URL"))
        .expect("RPC_URL is not set");

    let on_chain_data = OnChainData {
        contract_address,
        chain_id,
        rpc_url,
    };

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal, on_chain_data).await;
    ratatui::restore();
    result
}

async fn run(mut terminal: DefaultTerminal, on_chain_data: OnChainData) -> Result<()> {
    let services = Arc::new(Services::new(on_chain_data.clone()));

    let (intent_sender, mut intent_receiver) = mpsc::unbounded_channel::<AppIntent>();
    let (state_sender, state_receiver) = watch::channel(AppState::start(&on_chain_data));
    let (effects_sender, mut effects_receiver) = mpsc::unbounded_channel::<Vec<AppEffect>>();

    observe_keys(intent_sender.clone(), state_receiver.clone());

    // Render each new state
    let mut state_receiver_render = state_receiver.clone();
    tokio::spawn(async move {
        let first_state = state_receiver_render.borrow().clone();
        terminal
            .draw(|f| AppScreen::render(&first_state, &first_state.core, f, f.area()))
            .expect("First state should have rendered");

        while state_receiver_render.changed().await.is_ok() {
            let snapshot = state_receiver_render.borrow_and_update().clone();
            terminal
                .draw(|f| AppScreen::render(&snapshot, &snapshot.core, f, f.area()))
                .expect("State should have rendered");
        }
    });

    // Handle effects
    let effects_intent_sender = intent_sender.clone();
    tokio::spawn(async move {
        while let Some(effects) = effects_receiver.recv().await {
            for effect in effects {
                tokio::spawn(AppScreen::run(effect, services.clone(), effects_intent_sender.clone()));
            }
        }
    });

    // First intent to start splash screen's reducer
    intent_sender.send(splash::types::Intent::OnStart.into()).unwrap(); // TODO Error?

    // Observe intents and reduce them to new state or effects
    while let Some(intent) = intent_receiver.recv().await {
        let app_state = state_receiver.borrow().clone();
        let (new_state, effects) = AppScreen::reduce(&app_state, intent, &app_state.core);
        let running = new_state.core.running;

        state_sender.send_if_modified(|state| {
            if *state != new_state {
                *state = new_state;
                true
            } else {
                false
            }
        });

        if !running {
            break;
        }

        if !effects.is_empty() {
            effects_sender.send(effects)?;
        }
    }

    Ok(())
}

fn observe_keys(
    intent_sender: UnboundedSender<AppIntent>,
    state_receiver: watch::Receiver<AppState>,
) {
    std::thread::spawn(move || {
        loop {
            if let Ok(Event::Key(event)) = crossterm::event::read() {
                let current_state = state_receiver.borrow().clone();

                let top_screen = current_state
                    .screens
                    .first()
                    .expect("Received key but screen should exist")
                    .clone();
                let intent = match top_screen {
                    // TODO Boilerplate on_key
                    ScreenState::Splash(state) => {
                        splash::screen::SplashScreen::on_key(event, &state).map(|i| i.into())
                    }
                    ScreenState::Start(state) => {
                        start::screen::StartScreen::on_key(event, &state).map(|i| i.into())
                    }
                    ScreenState::Login(state) => {
                        login::screen::LoginScreen::on_key(event, &state).map(|i| i.into())
                    }
                }
                .or_else(|| AppScreen::on_key(event, &current_state));

                if let Some(intent) = intent {
                    intent_sender.send(intent).unwrap();
                }
            }
        }
    });
}
