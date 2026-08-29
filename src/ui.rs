//! ui.rs — Pure rendering for Termiban
//!
//! Highly customizable rendering:
//! * Rich task rows (priority, deadline, tags, description marker)
//! * Task detail, full task form, board manager, column forms

use crate::{
    app::{App, InputMode, TaskFormField},
    board::Priority,
    storage,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

// ---------------------------------------------------------------------------
// Public entry
// ---------------------------------------------------------------------------

pub fn ui(f: &mut Frame, app: &App) {
    let area = f.area();

    // Vertical: board fills, footer is fixed 3 lines.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    render_board(f, app, chunks[0]);
    render_footer(f, app, chunks[1]);

    // Popups on top (centered) — order matters: most specific last
    match app.input_mode {
        InputMode::Adding | InputMode::Editing => render_input_popup(f, app, area),
        InputMode::TaskForm => render_task_form_popup(f, app, area),
        InputMode::ViewingTask => render_task_detail_popup(f, app, area),
        InputMode::BoardManager => render_board_manager_popup(f, app, area),
        InputMode::AddingColumn | InputMode::RenamingColumn => render_column_popup(f, app, area),
        _ => {}
    }
    // Welcome/onboarding for empty board — highly visible starter menu
    if app.board.is_empty()
        && matches!(app.input_mode, InputMode::Normal)
        && !app.show_help
    {
        render_welcome_popup(f, app, area);
    }
    if app.show_help {
        render_help_popup(f, area);
    }
}

// ---------------------------------------------------------------------------
// Board columns — rich task rows
// ---------------------------------------------------------------------------

fn render_board(f: &mut Frame, app: &App, area: Rect) {
    // Empty board — show a big onboarding placeholder instead of columns
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

        let border_color = if is_selected_col {
            Color::Cyan
        } else {
            Color::DarkGray
        };
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
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if is_selected_task { "▶ " } else { "  " };
                let prefix_style = Style::default().fg(if is_selected_task {
                    Color::Cyan
                } else {
                    Color::DarkGray
                });

                // Priority icon with color
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

                // Title
                let title_span = Span::styled(task.title.clone(), base_style);

                // Deadline with overdue coloring
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
                        let dl_text = if task.is_overdue() {
                            format!(" ⚑{}", deadline)
                        } else {
                            format!(" ◷{}", deadline)
                        };
                        spans.push(Span::styled(dl_text, dl_style));
                    }
                }

                // Tags (first 2)
                if !task.tags.is_empty() {
                    let tag_text = format!(" #{}", task.tags.iter().take(2).cloned().collect::<Vec<_>>().join(" #"));
                    let tag_style = Style::default().fg(Color::Cyan);
                    spans.push(Span::styled(tag_text, tag_style));
                    if task.tags.len() > 2 {
                        spans.push(Span::styled(format!(" +{}", task.tags.len() - 2), Style::default().fg(Color::DarkGray)));
                    }
                }

                // Description marker
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
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
            )))];
            List::new(empty).block(block)
        } else {
            List::new(items).block(block)
        };

        f.render_widget(list, *chunk);
    }
}

// ---------------------------------------------------------------------------
// Footer — key hints updated for highly customizable board
// ---------------------------------------------------------------------------

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let footer_text = match app.input_mode {
        InputMode::Adding
        | InputMode::Editing
        | InputMode::AddingColumn
        | InputMode::RenamingColumn => vec![Line::from(vec![
            Span::styled(
                " Enter ",
                Style::default()
                    .bg(Color::Green)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" confirm  "),
            Span::styled(
                " Esc ",
                Style::default()
                    .bg(Color::Red)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" cancel  "),
            Span::styled(format!("  {} ", app.status), Style::default().fg(Color::Yellow)),
        ])],
        InputMode::TaskForm => vec![
            Line::from(vec![
                Span::styled(" Tab ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" next  "),
                Span::styled(" Shift+Tab ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" prev  "),
                Span::styled(" p ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" prio  "),
                Span::styled(" Enter ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
                Span::raw(" save  "),
                Span::styled(" Esc ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw(" cancel"),
            ]),
            Line::from(Span::styled(format!("  {} ", app.status), Style::default().fg(Color::DarkGray))),
        ],
        InputMode::ViewingTask => vec![Line::from(vec![
            Span::styled(" Esc/q ", Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" close  "),
            Span::styled(" E ", Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::raw(" edit  "),
            Span::styled(" p ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" prio  "),
            Span::styled(" d ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" del"),
        ])],
        InputMode::BoardManager => vec![
            Line::from(vec![
                Span::styled(" a ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
                Span::raw(" add  "),
                Span::styled(" r ", Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)),
                Span::raw(" rename  "),
                Span::styled(" d ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw(" del  "),
                Span::styled(" H/L ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw(" move  "),
                Span::styled(" Esc ", Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw(" close"),
            ]),
            Line::from(Span::styled(format!("  {} ", app.status), Style::default().fg(Color::DarkGray))),
        ],
        _ => vec![
            Line::from(vec![
                Span::styled(" h/l ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("col "),
                Span::styled("j/k", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" task  "),
                Span::styled(" H/L ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                Span::raw("move  "),
                Span::styled(" n ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
                Span::raw(" quick  "),
                Span::styled(" N ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
                Span::raw(" full  "),
                Span::styled(" Enter ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw(" view  "),
                Span::styled(" p ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" prio  "),
                Span::styled(" B ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(" board  "),
                Span::styled(" ? ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw(" help"),
            ]),
            Line::from(vec![
                Span::styled(format!("  ● {} ", app.board.title), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("— {} cols • ", app.board.column_count()), Style::default().fg(Color::DarkGray)),
                Span::styled(app.status.clone(), Style::default().fg(Color::DarkGray)),
            ]),
        ],
    };

    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(" Termiban v0.2.0 ", Style::default().fg(Color::Cyan)));

    let footer = Paragraph::new(footer_text)
        .block(footer_block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(footer, area);
}

// ---------------------------------------------------------------------------
// Popups
// ---------------------------------------------------------------------------

fn render_input_popup(f: &mut Frame, app: &App, area: Rect) {
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

fn render_column_popup(f: &mut Frame, app: &App, area: Rect) {
    let title = match app.input_mode {
        InputMode::AddingColumn => " Add Column — type name ",
        InputMode::RenamingColumn => " Rename Column — type new name ",
        _ => " Column ",
    };
    let popup_area = centered_rect(60, 20, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            title,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(Color::Cyan));

    let input_text = format!("{}█", app.input_buffer);
    let paragraph = Paragraph::new(input_text)
        .style(Style::default().fg(Color::White))
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, popup_area);
}

fn render_task_form_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(70, 60, area);
    f.render_widget(Clear, popup_area);

    let is_edit = app.task_form.editing_id.is_some();
    let title = if is_edit { " Edit Task — Full Form " } else { " New Task — Full Form " };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            title,
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(Color::Yellow));

    // Build form lines with focus highlight
    let form = &app.task_form;
    let mut lines: Vec<Line> = Vec::new();

    for field in [
        TaskFormField::Title,
        TaskFormField::Description,
        TaskFormField::Deadline,
        TaskFormField::Priority,
        TaskFormField::Tags,
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
                    "(no description)".to_string()
                } else if is_focused {
                    format!("{}█", form.description)
                } else {
                    let preview: String = form.description.chars().take(40).collect();
                    if form.description.len() > 40 { format!("{preview}…") } else { preview }
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
        };

        // Priority has extra help when focused
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

fn render_task_detail_popup(f: &mut Frame, app: &App, area: Rect) {
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

fn render_board_manager_popup(f: &mut Frame, app: &App, area: Rect) {
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

fn render_welcome_popup(f: &mut Frame, app: &App, area: Rect) {
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
        Line::from(Span::styled(
            "CREATE YOUR BOARD:",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
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
        Line::from(Span::styled(
            "HIGHLY CUSTOMIZABLE:",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from("  • Add any columns you want — Backlog, Review, Blocked, Archive…"),
        Line::from("  • Rename, reorder, or delete columns anytime via B"),
        Line::from("  • Tasks support deadline, priority, tags, description"),
        Line::from("  • Your board is saved as JSON — edit it directly if you like"),
        Line::from(""),
        Line::from(Span::styled(
            format!("File: {}", storage::data_path().display()),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'a' to add a column, 't' for template, or 'B' for manager",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Press 'q' to quit • '?' for help",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, popup_area);
}

fn render_help_popup(f: &mut Frame, area: Rect) {
    let help_area = centered_rect(78, 82, area);
    f.render_widget(Clear, help_area);

    let help_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Help — Termiban v0.2.0 (Highly Customizable) ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(Color::Cyan));

    let help_text = vec![
        Line::from(Span::styled("NAVIGATION", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  h/l or ←/→            Move between columns (B manager also)"),
        Line::from("  j/k or ↓/↑            Move between tasks"),
        Line::from(""),
        Line::from(Span::styled("TASK — QUICK", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  n / a                Quick add (title only)"),
        Line::from("  N (Shift+n)          Full add — deadline, priority, tags, description"),
        Line::from("  e                    Quick edit title"),
        Line::from("  E (Shift+e)          Full edit — all properties"),
        Line::from("  Enter / v            View detail (description, tags, deadline)"),
        Line::from("  p                    Cycle priority Low→Medium→High→Urgent"),
        Line::from("  d / x / Del          Delete task"),
        Line::from("  H / Shift+h          Move task to previous column"),
        Line::from("  L / Shift+l          Move task to next column"),
        Line::from(""),
        Line::from(Span::styled("TASK — PROPERTIES", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  In full form (N/E):"),
        Line::from("    Tab / Shift+Tab    Cycle field (Title→Desc→Deadline→Priority→Tags)"),
        Line::from("    p                  Cycle priority when Priority field focused"),
        Line::from("    Enter              Save • Esc: cancel"),
        Line::from("  Deadline format: YYYY-MM-DD (e.g. 2026-12-31), empty = none"),
        Line::from("  Tags: comma-separated (e.g. work, bug, urgent)"),
        Line::from("  Overdue deadlines shown in red ⚑, today in yellow"),
        Line::from(""),
        Line::from(Span::styled("BOARD — CUSTOMIZE YOUR BOARD", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  B / b                Open board manager"),
        Line::from("    a                  Add new column"),
        Line::from("    r                  Rename selected column"),
        Line::from("    d                  Delete column (can delete all → welcome)"),
        Line::from("    t                  Create from template (To Do/In Progress/Done)"),
        Line::from("    c                  Clear board (remove all columns)"),
        Line::from("    H/L or Shift+h/l   Move column left/right"),
        Line::from("    h/l or ←/→        Select column"),
        Line::from("  C (Shift+c)          Quick add column (no manager)"),
        Line::from("  R (Shift+r)          Quick rename column"),
        Line::from(""),
        Line::from(Span::styled("EMPTY BOARD — START FROM SCRATCH", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  When board is empty:"),
        Line::from("    a                  Add first column"),
        Line::from("    t                  Create template board"),
        Line::from("    B                  Open manager to build custom board"),
        Line::from(""),
        Line::from(Span::styled("GENERAL", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  ?                    Toggle this help"),
        Line::from("  q / Esc              Quit (auto-saves) • In popups: cancel/close"),
        Line::from(""),
        Line::from(Span::styled("PERSISTENCE & CUSTOM BOARD", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(format!("  Auto-saved to: {}", storage::data_path().display())),
        Line::from("  Override with $TERMIBAN_DATA_PATH (portable)"),
        Line::from("  JSON: { title, columns:[{name,tasks:[{id,title,description,deadline,priority,tags,created_at}]}], next_id }"),
        Line::from("  Customize: edit board.json directly or use B manager; add any columns you want"),
        Line::from(""),
        Line::from(Span::styled("TIPS", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from("  • Example: Add columns 'Backlog','Review','Blocked' via B → a"),
        Line::from("  • Example: Set Urgent + tomorrow deadline for critical tasks"),
        Line::from("  • Works on Linux, macOS, Windows — single binary `cargo build --release`"),
        Line::from(""),
        Line::from(Span::styled("Press any key to close", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))),
    ];

    let help_para = Paragraph::new(help_text)
        .block(help_block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(help_para, help_area);
}

// ---------------------------------------------------------------------------
// Layout helper — centered rectangle
// ---------------------------------------------------------------------------

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
