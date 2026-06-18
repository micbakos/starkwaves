mod app;
mod types;
mod onboard;

use crate::app::screen::State;
use crate::types::effect::Effect;
use crate::types::intent::Intent;
use crate::types::screen::Screen;
use crate::types::state::ScreenState;
use color_eyre::Result;
use crossterm::event::Event;
use ratatui::layout::Constraint::{Length, Percentage};
use ratatui::layout::Layout;
use ratatui::prelude::Direction;
use ratatui::{DefaultTerminal, Frame};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{mpsc, watch};

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal).await;
    ratatui::restore();
    result
}

async fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let (intent_sender, mut intent_receiver) = mpsc::unbounded_channel::<Intent>();
    let (state_sender, state_receiver) = watch::channel(State::start());
    let (effects_sender, mut effects_receiver) = mpsc::unbounded_channel::<Vec<Effect>>();

    spawn_keystrokes_reader(intent_sender.clone(), state_receiver.clone());

    // Render each new state
    let mut state_receiver_render = state_receiver.clone();
    tokio::spawn(async move {
        let first_state = state_receiver_render.borrow().clone();
        terminal.draw(|f| render(&first_state, f)).unwrap(); // TODO error propagation

        while state_receiver_render.changed().await.is_ok() {
            let snapshot = state_receiver_render.borrow_and_update().clone();
            terminal.draw(|f| render(&snapshot, f)).unwrap(); // TODO error propagation
        }
    });

    tokio::spawn(async move {
        while let Some(effects) = effects_receiver.recv().await {

        }
    });

    // Observe intents and reduce them to new state or effects
    while let Some(intent) =intent_receiver.recv().await {
        let (new_state, effects) = reduce(&state_receiver.borrow().clone(), intent);

        state_sender.send_if_modified(|state| {
            if *state != new_state {
                *state = new_state;
                true
            } else {
                false
            }
        });

        if !effects.is_empty() {
            effects_sender.send(effects).unwrap();
        }
    }

    Ok(())
}

fn reduce(state: &State, intent: Intent) -> (State, Vec<Effect>) {
    match intent {
        Intent::App(intent) => {
            let (state, effects) = app::screen::reduce(state, intent);

            (state, effects.into_iter().map(|e| Effect::App(e)).collect())
        },
        Intent::Start(intent) => {
            // TODO: Boilerplate
            let mut state = state.clone();
            let screen = state.screens.first_mut()
                .expect("Received intent but no screen exists in stack.");
            let start_state = screen.as_start_mut()
                .expect("Received start intent but no start screen.");

            let (screen_state, effects) = onboard::start::screen::StartScreen::reduce(
                start_state,
                intent,
                &state.core
            );

            state.screens[0] = ScreenState::Start(screen_state);

            (state, effects.into_iter().map(|e| Effect::Start(e)).collect())
        }
    }
}

fn render(state: &State, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Percentage(100),
            Length(1),
        ])
        .split(frame.area());

    let screen = state.screens.first().expect("No screen exists to render");

    match screen {
        // TODO: boilerplate
        ScreenState::Start(start_screen) => {
            onboard::start::screen::StartScreen::render(
                start_screen,
                &state.core,
                frame,
                layout[0],
            )
        }
    }

    app::screen::AppScreen::render(state, &state.core, frame, layout[1]);
}

async fn run_effect(effect: Effect, intent_sender: UnboundedSender<Intent>) {
    match effect {
        Effect::App(_) => {}
        Effect::Start(_) => {}
    }
}

fn spawn_keystrokes_reader(
    intent_sender: UnboundedSender<Intent>,
    state_receiver: watch::Receiver<State>
) {
    std::thread::spawn(move || loop {
        if let Ok(Event::Key(event)) = crossterm::event::read() {
            let current_state = state_receiver.borrow().clone();

            let intent = if let Some(top_screen) = current_state.screens.first() {
                match top_screen {
                    ScreenState::Start(_) => {
                        onboard::start::screen::StartScreen::on_key(event).map(|i| Intent::Start(i))
                    }
                }
            } else {
                app::screen::AppScreen::on_key(event).map(|i| Intent::App(i))
            };

            if let Some(intent) = intent {
                intent_sender.send(intent).unwrap();
            }
        }
    });
}
