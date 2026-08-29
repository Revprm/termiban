//! ui/popups/welcome.rs — Empty-board onboarding

use crate::app::App;
use crate::storage;
use crate::ui::layout::centered_rect;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_welcome_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(70, 60, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " ✨ Welcome to Termiban — Start From Scratch ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(Color::Yellow));

    let lines = vec![
        Line::from(Span::styled(
            format!("Your board \"{}\" is empty", app.board.title),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "No columns yet — make it truly yours!",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(Span::styled("CREATE YOUR BOARD:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("  a ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::raw("  Add first column (e.g. \"Ideas\", \"To Do\")"),
        ]),
        Line::from(vec![
            Span::styled("  t ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::raw("  Create from template (To Do → In Progress → Done)"),
        ]),
        Line::from(vec![
            Span::styled("  B ", Style::default().bg(Color::Magenta).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw("  Open board manager for full customization"),
        ]),
        Line::from(""),
        Line::from(Span::styled("HIGHLY CUSTOMIZABLE:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  • Add any columns you want — Backlog, Review, Blocked, Archive…"),
        Line::from("  • Rename, reorder, or delete columns anytime via B"),
        Line::from("  • Tasks support deadline, priority, tags, description"),
        Line::from("  • Your board is saved as JSON — edit it directly if you like"),
        Line::from(""),
        Line::from(Span::styled(format!("File: {}", storage::data_path().display()), Style::default().fg(Color::DarkGray))),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'a' to add a column, 't' for template, or 'B' for manager",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("Press 'q' to quit • '?' for help", Style::default().fg(Color::DarkGray))),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, popup_area);
}
