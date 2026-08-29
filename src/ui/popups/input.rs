//! ui/popups/input.rs — Quick add/edit title popup

use crate::app::{App, InputMode};
use crate::ui::layout::centered_rect;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use ratatui::layout::Rect;

pub fn render_input_popup(f: &mut Frame, app: &App, area: Rect) {
    let (title, is_adding) = match app.input_mode {
        InputMode::Adding => (" Quick Add — type title (N for full) ", true),
        InputMode::Editing => (" Quick Edit — update title (E for full) ", false),
        _ => (" Input ", true),
    };

    let popup_area = centered_rect(60, 20, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            title,
            Style::default().fg(if is_adding { Color::Green } else { Color::Yellow }).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(if is_adding { Color::Green } else { Color::Yellow }));

    let input_text = format!("{}█", app.input_buffer);
    let paragraph = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White))
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, popup_area);
}
