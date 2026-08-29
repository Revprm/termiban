//! ui/mod.rs — Rendering dispatcher (highly modular)
//!
//! Delegates to focused submodules: board, footer, popups, layout.

pub mod board;
pub mod footer;
pub mod layout;
pub mod popups;

use crate::app::{App, InputMode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub fn ui(f: &mut Frame, app: &App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    board::render_board(f, app, chunks[0]);
    footer::render_footer(f, app, chunks[1]);

    match app.input_mode {
        InputMode::Adding | InputMode::Editing => popups::input::render_input_popup(f, app, area),
        InputMode::TaskForm => popups::task_form::render_task_form_popup(f, app, area),
        InputMode::ViewingTask => popups::task_detail::render_task_detail_popup(f, app, area),
        InputMode::BoardManager => popups::board_manager::render_board_manager_popup(f, app, area),
        InputMode::AddingColumn | InputMode::RenamingColumn => popups::column::render_column_popup(f, app, area),
        _ => {}
    }
    if app.board.is_empty() && matches!(app.input_mode, InputMode::Normal) && !app.show_help {
        popups::welcome::render_welcome_popup(f, app, area);
    }
    if app.show_help {
        popups::help::render_help_popup(f, area);
    }
}
