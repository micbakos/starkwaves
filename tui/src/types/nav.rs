use std::{collections::VecDeque, iter::once};

use crate::types::{AppState, ScreenIntent, ScreenKind, ScreenState, screens_on_start};

#[derive(Debug, Clone, PartialEq, Eq, enum_as_inner::EnumAsInner)]
pub enum NavCommand {
    Push(ScreenState),
    Replace(ScreenState),
    Pop,
    PopTo(ScreenKind),
    ResetTo(ScreenState),
}

impl NavCommand {
    pub fn handle(&self, state: &mut AppState) -> Option<ScreenIntent> {
        let mut screen_intent: Option<ScreenIntent> = None;
        match self {
            NavCommand::Push(screen_state) => {
                screen_intent = screens_on_start(screen_state);
                state.screens.insert(0, screen_state.clone());
            }
            NavCommand::Replace(screen_state) => {
                screen_intent = screens_on_start(screen_state);
                state.screens[0] = screen_state.clone();
            }
            NavCommand::Pop => {
                if state.screens.len() == 1 {
                    state.core.running = false;
                } else {
                    state.screens.pop_front();
                }
            }
            NavCommand::PopTo(screen_kind) => {
                let index = state
                    .screens
                    .iter()
                    .position(|screen_state| screen_state.kind() == *screen_kind)
                    .unwrap_or_else(|| {
                        panic!(
                            "Expected to pop to {}, but such screen doesn't exist.",
                            screen_kind
                        )
                    });

                if index == 0 {
                    state.core.running = false;
                } else {
                    let _ = state.screens.drain(..=index);
                }
            }
            NavCommand::ResetTo(screen_state) => {
                screen_intent = screens_on_start(screen_state);
                state.screens = VecDeque::from_iter([screen_state.clone()]);
            }
        }
        screen_intent
    }
}
