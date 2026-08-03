use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Clear, Paragraph},
};
use strum::EnumMessage;

pub fn render_popup<I: EnumMessage + PartialEq + std::fmt::Debug>(
    frame: &mut Frame,
    area: Rect,
    title: Option<impl Into<String>>,
    message: impl Into<String>,
    selected_action: &I,
    actions: &[I],
) {
    let mut popup_block = Block::bordered();
    if let Some(title) = title {
        popup_block = popup_block.title(title.into());
    }
    let popup_area = area.centered(Constraint::Percentage(60), Constraint::Percentage(20));
    frame.render_widget(Clear, popup_area);
    let inner_area = popup_block.inner(popup_area);
    frame.render_widget(popup_block, popup_area);

    let popup_layout = Layout::default()
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .split(inner_area);

    let message = Paragraph::new(message.into()).centered();
    let [_, message_area, _] = Layout::default()
        .constraints([Constraint::Fill(1), Constraint::Min(1), Constraint::Fill(1)])
        .areas(popup_layout[0]);

    frame.render_widget(message, message_area);

    let buttons_layout =
        Layout::horizontal(actions.into_iter().map(|_| Constraint::Fill(1))).split(popup_layout[1]);

    let selected_style = Style::default().reversed();
    let normal_style = Style::default();

    for (index, action) in actions.iter().enumerate() {
        let label = action
            .get_message()
            .expect(format!("Expected {:?} variant to implement get_message", action).as_str());

        let style = if selected_action == action {
            selected_style
        } else {
            normal_style
        };

        let line = Line::raw(label).centered().style(style);
        frame.render_widget(line, buttons_layout[index]);
    }
}
