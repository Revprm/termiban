//! ui/popups/help.rs — Help overlay

use crate::storage;
use crate::ui::layout::centered_rect;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_help_popup(f: &mut Frame, area: Rect) {
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
        Line::from("  • Linux-only — single binary `cargo build --release`"),
        Line::from(""),
        Line::from(Span::styled("Press any key to close", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))),
    ];

    let help_para = Paragraph::new(help_text)
        .block(help_block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    f.render_widget(help_para, help_area);
}
