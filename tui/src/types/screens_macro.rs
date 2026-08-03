#[macro_export]
macro_rules! screens {
    (
        $( $screen_name:ident => $screen_path:path ),+
    ) => {
        #[derive(derive_more::From)]
        pub enum ScreenIntent {
            $(
                $screen_name(<$screen_path as crate::types::screen::Screen>::Intent),
            )+
        }

        $(
            impl From<<$screen_path as crate::types::screen::Screen>::Intent> for crate::types::AppIntent {
                fn from(value: <$screen_path as crate::types::screen::Screen>::Intent) -> Self {
                    let screen_intent: ScreenIntent = value.into();
                    screen_intent.into()
                }
            }
        )+

        #[derive(Debug, Clone, PartialEq, Eq, EnumAsInner, derive_more::From)]
        pub enum ScreenState {
            $(
                $screen_name(<$screen_path as crate::types::screen::Screen>::State),
            )+
        }

        impl ScreenState {
            pub fn kind(&self) -> ScreenKind {
                match self {
                    $( ScreenState::$screen_name(_) => ScreenKind::$screen_name, )+
                }
            }
        }

        #[derive(Debug, Clone, PartialEq, Eq, EnumAsInner, derive_more::Display)]
        pub enum ScreenKind {
            $(
                #[display("{} screen", stringify!($screen_name))]
                $screen_name,
             )+
        }

        #[derive(derive_more::From)]
        pub enum ScreenEffect {
            $(
                $screen_name(<$screen_path as crate::types::screen::Screen>::Effect),
            )+
        }

        $(
            impl From<<$screen_path as crate::types::screen::Screen>::Effect> for crate::types::AppEffect {
                fn from(value: <$screen_path as crate::types::screen::Screen>::Effect) -> Self {
                    let screen_effect: ScreenEffect = value.into();
                    screen_effect.into()
                }
            }
        )+

        pub fn screens_on_push_effect(state: &ScreenState) -> Option<AppEffect> {
            match (state) {
                $(
                    ScreenState::$screen_name(_) => <$screen_path as crate::types::screen::Screen>::on_push_effect().map(Into::into),
                )+
            }
        }

        pub fn screens_on_pop_effect(state: &ScreenState) -> Option<AppEffect> {
            match (state) {
                $(
                    ScreenState::$screen_name(_) => <$screen_path as crate::types::screen::Screen>::on_pop_effect().map(Into::into),
                )+
            }
        }

        pub fn screens_on_key(state: &ScreenState, key: crossterm::event::KeyEvent) -> Option<ScreenIntent> {
            match state {
                $(
                    ScreenState::$screen_name(screen_state) => <$screen_path as crate::types::screen::Screen>::on_key(key, screen_state).map(Into::into),
                )+
            }
        }

        pub fn screens_reduce(intent: ScreenIntent, top_screen_state: ScreenState, core_state: &CoreState) -> (ScreenState, Vec<AppEffect>) {
            match intent {
                $(
                    ScreenIntent::$screen_name(screen_intent) => {
                        if let ScreenState::$screen_name(state) = top_screen_state {
                            let (screen_state, effects) = <$screen_path as crate::types::screen::Screen>::reduce(
                                &state,
                                screen_intent,
                                core_state
                            );

                            (screen_state.into(), effects)
                        } else {
                            (top_screen_state, vec![])
                        }
                    },
                )+
            }
        }

        pub fn screens_render(screen_state: &ScreenState, core_state: &CoreState, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
            match screen_state {
                $(
                    ScreenState::$screen_name(state) => {
                        <$screen_path as crate::types::screen::Screen>::render(state, core_state, frame, area);
                    }
                )+
            }
        }

        pub async fn screens_run(
            effect: ScreenEffect,
            services: std::sync::Arc<crate::app::services::Services>,
            intents: tokio::sync::mpsc::UnboundedSender<AppIntent>
        ) -> Result<(), tokio::sync::mpsc::error::SendError<AppIntent>> {
            match effect {
                $(
                    ScreenEffect::$screen_name(effect) => <$screen_path as crate::types::screen::Screen>::run(
                        effect,
                        services,
                        intents
                    ).await?,
                )+
            }

            Ok(())
        }
    };
}
