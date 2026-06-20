use crate::app::core::CoreState;
use crate::types::screen::Screen;
use crossterm::event::{KeyCode, KeyEvent};
use enum_as_inner::EnumAsInner;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use strum::{EnumCount, FromRepr, VariantArray};
use tokio::sync::mpsc::UnboundedSender;

pub struct StartScreen;

#[derive(Copy, Debug, Clone, PartialEq, Eq, EnumAsInner, EnumCount, FromRepr, VariantArray)]
#[repr(u8)]
enum Menu {
    Start = 0,
    Quit = 1,
}

impl Default for Menu {
    fn default() -> Self { Self::Start }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    selected_button: Menu,
}

impl State {
    pub fn new() -> Self {
        Self {
            selected_button: Menu::default(),
        }
    }

    pub fn press_down(&mut self) {
        let next = (self.selected_button as u8 + 1) % Menu::COUNT as u8;
        self.selected_button = Menu::from_repr(next).unwrap();
    }

    pub fn press_up(&mut self) {
        let count = Menu::COUNT as u8;
        let prev = (self.selected_button as u8 + count - 1) % count;
        self.selected_button = Menu::from_repr(prev).unwrap();
    }
}

pub enum Intent {
    OnPressDown,
    OnPressUp,
    OnSelect,
}

pub enum Effect {

}

impl Screen for StartScreen {
    type Intent = Intent;
    type Effect = Effect;
    type State = State;

    fn reduce(state: &Self::State, intent: Self::Intent, core: &CoreState) -> (Self::State, Vec<crate::types::effect::Effect>) {
        let mut new_state = state.clone();
        let mut effects: Vec<crate::types::effect::Effect> = vec![];
        match intent {
            Intent::OnPressDown => new_state.press_down(),
            Intent::OnPressUp => new_state.press_up(),
            Intent::OnSelect => {
                match state.selected_button {
                    Menu::Start => {

                    }
                    Menu::Quit => {
                        effects.push(crate::app::screen::Effect::RequestQuit.into())
                    }
                }
            }
        };

        (new_state, effects)
    }

    fn render(state: &Self::State, core: &CoreState, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .border_style(Style::default().fg(Color::Magenta))
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        frame.render_widget(block, area);

        let lines = Menu::VARIANTS.into_iter().map(|menu| {
            let label = match menu {
                Menu::Start => "Start Game",
                Menu::Quit => "Quit"
            };

            let line_style = if state.selected_button == *menu {
                Style::default().reversed()
            } else {
                Style::default()
            };

            Line::raw(label).style(line_style).centered()
        }).collect::<Vec<_>>();

        let [buttons_area] = Layout::vertical([Constraint::Length(lines.len() as u16)])
            .flex(Flex::Center)
            .areas(inner_area);

        frame.render_widget(Paragraph::new(lines), buttons_area);
    }

    fn on_key(key: KeyEvent) -> Option<Self::Intent> {
        match key.code {
            KeyCode::Up => Some(Intent::OnPressUp),
            KeyCode::Down => Some(Intent::OnPressDown),
            KeyCode::Enter => Some(Intent::OnSelect),
            _ => None,
        }
    }

    async fn run(effect: Self::Effect, intents: UnboundedSender<crate::types::intent::Intent>) {

    }
}