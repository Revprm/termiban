//! ui/popups/task_detail.rs — Task detail view

use crate::app::App;
use crate::board::Priority;
use crate::ui::layout::centered_rect;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_task_detail_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(70, 60, area);
    f.render_widget(Clear, popup_area);

    let task = app.selected_task();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Task Detail — press E to edit, p to cycle priority ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(Color::Cyan));

    let lines = if let Some(t) = task {
        let pri_color = match t.priority {
            Priority::Low => Color::DarkGray,
            Priority::Medium => Color::White,
            Priority::High => Color::Yellow,
            Priority::Urgent => Color::Red,
        };
        let deadline_line = if let Some(d) = &t.deadline {
            let dl_style = if t.is_overdue() {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else if t.is_due_today() {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let overdue = if t.is_overdue() { " (OVERDUE)" } else if t.is_due_today() { " (TODAY)" } else { "" };
            Line::from(vec![
                Span::styled("Deadline: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}{}", d, overdue), dl_style),
            ])
        } else {
            Line::from(vec![
                Span::styled("Deadline: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("(none)", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
            ])
        };

        vec![
            Line::from(vec![
                Span::styled("Title:    ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(t.title.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("ID:       ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}", t.id), Style::default().fg(Color::DarkGray)),
                Span::raw("   "),
                Span::styled("Priority: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} {}", t.priority.icon(), t.priority), Style::default().fg(pri_color).add_modifier(Modifier::BOLD)),
            ]),
            deadline_line,
            Line::from(vec![
                Span::styled("Tags:     ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(
                    if t.tags.is_empty() { "(none)".to_string() } else { t.tags_display() },
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Created:  ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(t.created_at.clone().unwrap_or_else(|| "(unknown)".into()), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(""),
            Line::from(Span::styled("Description:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled(
                if t.description.is_empty() { "(no description)".to_string() } else { t.description.clone() },
                Style::default().fg(Color::White),
            )),
            Line::from(""),
            Line::from(Span::styled("— Press Enter/Esc to close —", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))),
        ]
    } else {
        vec![Line::from(Span::styled("(no task selected)", Style::default().fg(Color::DarkGray)))]
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, popup_area);
}
