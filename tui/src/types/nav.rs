use std::collections::VecDeque;

use log::debug;

use crate::{
    app::types::Effect::RequestSettleNav,
    types::{
        AppEffect, AppState, ScreenKind, ScreenState, screens_name, screens_on_pop_effect,
        screens_on_push_effect,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, enum_as_inner::EnumAsInner)]
pub enum NavCommand {
    Push(ScreenState),
    Replace(ScreenState),
    Pop,
    PopTo(ScreenKind),
    ResetTo(ScreenState),
}

impl NavCommand {
    pub fn prepare_effects(&self, state: &AppState) -> Vec<AppEffect> {
        let mut effects: Vec<AppEffect> = vec![];
        let mut settle = SettleNavCommand::new();
        match self {
            NavCommand::Push(screen) => {
                if let Some(top_screen) = state.screens.front()
                    && let Some(pop_effect) = screens_on_pop_effect(top_screen)
                {
                    effects.push(pop_effect);
                }
                settle.push = Some(screen.clone());
            }
            NavCommand::Replace(screen) => {
                if let Some(top_screen) = state.screens.front()
                    && let Some(pop_effect) = screens_on_pop_effect(top_screen)
                {
                    effects.push(pop_effect);
                }

                settle.pop = 0..1;
                settle.push = Some(screen.clone());
            }
            NavCommand::Pop => {
                if let Some(top_screen) = state.screens.front()
                    && let Some(pop_effect) = screens_on_pop_effect(top_screen)
                {
                    effects.push(pop_effect);
                }
                settle.pop = 0..1;
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

                settle.pop = 0..index;
                for i in settle.pop.clone() {
                    if let Some(pop_effect) = screens_on_pop_effect(&state.screens[i]) {
                        effects.push(pop_effect);
                    }
                }
            }
            NavCommand::ResetTo(screen) => {
                settle.pop = 0..state.screens.len();
                for screen in state.screens.iter() {
                    if let Some(pop_effect) = screens_on_pop_effect(&screen) {
                        effects.push(pop_effect);
                    }
                }
                settle.push = Some(screen.clone());
            }
        }
        effects.push(RequestSettleNav(settle).into());
        effects
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettleNavCommand {
    pub push: Option<ScreenState>,
    pub pop: std::ops::Range<usize>,
}

impl SettleNavCommand {
    pub fn new() -> Self {
        Self {
            push: None,
            pop: (0..0),
        }
    }

    pub fn settle(&self, state: &mut AppState) -> Vec<AppEffect> {
        let mut effects: Vec<AppEffect> = vec![];

        if self.push.is_none() && self.pop == (0..state.screens.len()) {
            state.core.running = false;
            return effects;
        }

        if !self.pop.is_empty() {
            state.screens.drain(self.pop.clone());
        }

        if let Some(push_screen) = &self.push {
            state.screens.push_front(push_screen.clone());
            if let Some(push_effect) = screens_on_push_effect(push_screen) {
                effects.push(push_effect);
            }
        }

        effects
    }
}
