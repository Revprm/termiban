//! ui/footer.rs — Footer key hints

use crate::app::{App, InputMode};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let footer_text = match app.input_mode {
        InputMode::Adding | InputMode::Editing | InputMode::AddingColumn | InputMode::RenamingColumn => vec![Line::from(vec![
            Span::styled(" Enter ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::raw(" confirm  "),
            Span::styled(" Esc ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
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
                Span::raw(" delete  "),
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
