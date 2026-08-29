//! ui/popups/column.rs — Add/rename column popup

use crate::app::{App, InputMode};
use crate::ui::layout::centered_rect;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_column_popup(f: &mut Frame, app: &App, area: Rect) {
    let title = match app.input_mode {
        InputMode::AddingColumn => " Add Column — type name ",
        InputMode::RenamingColumn => " Rename Column — type new name ",
        _ => " Column ",
    };
    let popup_area = centered_rect(60, 20, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
        .border_style(Style::default().fg(Color::Cyan));

    let input_text = format!("{}█", app.input_buffer);
    let paragraph = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White))
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, popup_area);
}
