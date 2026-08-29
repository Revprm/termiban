//! ui/popups/task_form.rs — Full task form (all properties)

use crate::app::{App, TaskFormField};
use crate::ui::layout::centered_rect;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_task_form_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(70, 70, area);
    f.render_widget(Clear, popup_area);

    let is_edit = app.task_form.editing_id.is_some();
    let title = if is_edit { " Edit Task — Full Form " } else { " New Task — Full Form " };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(title, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
        .border_style(Style::default().fg(Color::Yellow));

    let form = &app.task_form;
    let mut lines: Vec<Line> = Vec::new();

    for field in [
        TaskFormField::Title,
        TaskFormField::Description,
        TaskFormField::Deadline,
        TaskFormField::Priority,
        TaskFormField::Tags,
        TaskFormField::Url,
    ] {
        let is_focused = form.field == field;
        let focus_marker = if is_focused { "▶ " } else { "  " };
        let focus_style = if is_focused {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let label_style = if is_focused {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let value_style = if is_focused {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let value = match field {
            TaskFormField::Title => {
                if form.title.is_empty() && !is_focused {
                    "(empty)".to_string()
                } else if is_focused {
                    format!("{}█", form.title)
                } else {
                    form.title.clone()
                }
            }
            TaskFormField::Description => {
                if form.description.is_empty() && !is_focused {
                    "(no description — long text supported, wraps)".to_string()
                } else if is_focused {
                     let len = form.description.chars().count();
                    let hint = if len > 200 {
                        format!(" ({} chars, wraps) ", len)
                    } else if len > 0 {
                        format!(" ({} chars) ", len)
                    } else {
                        String::new()
                    };
                    format!("{}{}█", form.description, hint)
                } else {
                    // Preview for not focused: show first 60 chars + length
                    let len = form.description.chars().count();
                    if len <= 60 {
                        form.description.clone()
                    } else {
                        let preview: String = form.description.chars().take(60).collect();
                        format!("{preview}… ({} chars)", len)
                    }
                }
            }
            TaskFormField::Deadline => {
                let v = if form.deadline.is_empty() { "(none)".to_string() } else { form.deadline.clone() };
                if is_focused { format!("{}█", form.deadline) } else { v }
            }
            TaskFormField::Priority => form.priority.to_string(),
            TaskFormField::Tags => {
                let v = if form.tags.is_empty() { "(none)".to_string() } else { form.tags.clone() };
                if is_focused { format!("{}█", form.tags) } else { v }
            }
            TaskFormField::Url => {
                if form.url.is_empty() && !is_focused {
                    "(none — e.g. https://...)".to_string()
                } else if is_focused {
                    format!("{}█", form.url)
                } else {
                    form.url.clone()
                }
            }
        };

        let label = if field == TaskFormField::Priority && is_focused {
            format!("{} (press 'p' to cycle)", field.label())
        } else {
            field.label().to_string()
        };

        lines.push(Line::from(vec![
            Span::styled(focus_marker, focus_style),
            Span::styled(format!("{:<26}", label), label_style),
            Span::styled(value, value_style),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(" Tab/Shift+Tab ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw(" switch  "),
        Span::styled(" p ", Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw(" prio  "),
        Span::styled(" Enter ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::raw(" save  "),
        Span::styled(" Esc ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" cancel"),
    ]));
    if !app.status.is_empty() {
        lines.push(Line::from(Span::styled(format!("  {}", app.status), Style::default().fg(Color::Yellow))));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, popup_area);
}
