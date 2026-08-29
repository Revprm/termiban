//! ui/popups/board_manager.rs — Board manager (customize columns)

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

pub fn render_board_manager_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(60, 50, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(" Board Manager — {} ({} cols) ", app.board.title, app.board.column_count()),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(Color::Cyan));

    let mut lines = vec![
        Line::from(Span::styled(format!("Board: {}", app.board.title), Style::default().fg(Color::White).add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(format!("File: {}", storage::data_path().display()), Style::default().fg(Color::DarkGray))),
        Line::from(""),
        Line::from(Span::styled("Columns:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
    ];

    if app.board.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (no columns yet — create your first!)",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )));
    } else {
        for (idx, col) in app.board.columns.iter().enumerate() {
            let is_sel = idx == app.selected_col;
            let marker = if is_sel { "▶ " } else { "  " };
            let style = if is_sel {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD).bg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(vec![
                Span::styled(marker, style),
                Span::styled(format!("{:>2}. {} ({} tasks)", idx + 1, col.name, col.tasks.len()), style),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" a ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw(" add  "),
        Span::styled(" r ", Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw(" rename  "),
        Span::styled(" d ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" delete  "),
        Span::styled(" H/L ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::raw(" move"),
    ]));
    lines.push(Line::from(vec![
        Span::styled(" t ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw(" template  "),
        Span::styled(" c ", Style::default().bg(Color::Red).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw(" clear board"),
    ]));
    lines.push(Line::from(Span::styled(" h/l: select column • Esc/q: close", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, popup_area);
}
