//! ui/board.rs — Board columns rendering

use crate::{
    app::{App, InputMode},
    board::Priority,
    storage,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use ratatui::layout::{Alignment, Rect};

pub fn render_board(f: &mut Frame, app: &App, area: Rect) {
    if app.board.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                " No Board Yet ",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ))
            .border_style(Style::default().fg(Color::Yellow))
            .padding(ratatui::widgets::Padding::uniform(2));

        let empty_text = vec![
            Line::from(Span::styled(
                "Your board is empty — start from scratch!",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Create your own custom board:",
                Style::default().fg(Color::Yellow),
            )),
            Line::from("  • Press 'a'  — Add your first column (e.g. \"To Do\")"),
            Line::from("  • Press 't'  — Create from template (To Do / In Progress / Done)"),
            Line::from("  • Press 'B'  — Open board manager for full customization"),
            Line::from(""),
            Line::from(Span::styled(
                format!("Board: {} • File: {}", app.board.title, storage::data_path().display()),
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "Tip: You can delete all columns and rebuild — Termiban now supports empty boards!",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            )),
        ];

        let paragraph = Paragraph::new(empty_text)
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        return;
    }

    let col_count = app.board.column_count().max(1);
    let col_constraints: Vec<Constraint> = (0..col_count)
        .map(|_| Constraint::Percentage((100 / col_count) as u16))
        .collect();

    let col_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(col_constraints)
        .split(area);

    for (idx, (column, chunk)) in app.board.columns.iter().zip(col_chunks.iter()).enumerate() {
        let is_selected_col =
            idx == app.selected_col && matches!(app.input_mode, InputMode::Normal | InputMode::BoardManager);

        let border_color = if is_selected_col { Color::Cyan } else { Color::DarkGray };
        let border_style = if is_selected_col {
            Style::default().fg(border_color).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(border_color)
        };

        let title = format!(" {} ({}) ", column.name, column.tasks.len());
        let title_style = match idx % 4 {
            0 => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            1 => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            2 => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(title, title_style))
            .border_style(border_style)
            .padding(ratatui::widgets::Padding::uniform(1));

        let items: Vec<ListItem> = column
            .tasks
            .iter()
            .enumerate()
            .map(|(row_idx, task)| {
                let is_selected_task = is_selected_col && row_idx == app.selected_row;
                let base_style = if is_selected_task {
                    Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if is_selected_task { "▶ " } else { "  " };
                let prefix_style = Style::default().fg(if is_selected_task { Color::Cyan } else { Color::DarkGray });

                let (pri_icon, pri_color) = match task.priority {
                    Priority::Low => ("·", Color::DarkGray),
                    Priority::Medium => ("●", Color::White),
                    Priority::High => ("▲", Color::Yellow),
                    Priority::Urgent => ("‼", Color::Red),
                };
                let pri_style = if is_selected_task {
                    Style::default().fg(pri_color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(pri_color)
                };

                let title_span = Span::styled(task.title.clone(), base_style);

                let mut spans = vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled(format!("{} ", pri_icon), pri_style),
                    title_span,
                ];

                if let Some(deadline) = &task.deadline {
                    if !deadline.is_empty() {
                        let dl_style = if task.is_overdue() {
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                        } else if task.is_due_today() {
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        };
                        let dl_text = if task.is_overdue() { format!(" ⚑{}", deadline) } else { format!(" ◷{}", deadline) };
                        spans.push(Span::styled(dl_text, dl_style));
                    }
                }

                if !task.tags.is_empty() {
                    let tag_text = format!(" #{}", task.tags.iter().take(2).cloned().collect::<Vec<_>>().join(" #"));
                    let tag_style = Style::default().fg(Color::Cyan);
                    spans.push(Span::styled(tag_text, tag_style));
                    if task.tags.len() > 2 {
                        spans.push(Span::styled(format!(" +{}", task.tags.len() - 2), Style::default().fg(Color::DarkGray)));
                    }
                }

                if !task.description.is_empty() {
                    spans.push(Span::styled(" ≡", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)));
                }

                let line = Line::from(spans);
                ListItem::new(line).style(base_style)
            })
            .collect();

        let list = if items.is_empty() {
            let empty = vec![ListItem::new(Line::from(Span::styled(
                "  (empty) — press 'n' / 'N' to add",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            )))];
            List::new(empty).block(block)
        } else {
            List::new(items).block(block)
        };

        f.render_widget(list, *chunk);
    }
}
