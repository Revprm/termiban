//! app/handlers/mod.rs — Key dispatch

mod board_manager;
mod insert;
mod normal;
mod task_form;
mod view;

use crate::app::state::{App, InputMode};
use crossterm::event::KeyEvent;

impl App {
    /// Handle a key event. Returns `true` if the app should quit.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.show_help {
            match key.code {
                crossterm::event::KeyCode::Char('?')
                | crossterm::event::KeyCode::Char('q')
                | crossterm::event::KeyCode::Esc => {
                    self.show_help = false;
                    self.status = "Help closed".to_string();
                }
                _ => self.show_help = false,
            }
            return false;
        }

        match self.input_mode.clone() {
            InputMode::Normal => self.handle_key_normal(key),
            InputMode::Adding | InputMode::Editing => self.handle_key_insert(key),
            InputMode::TaskForm => self.handle_key_task_form(key),
            InputMode::ViewingTask => self.handle_key_viewing(key),
            InputMode::BoardManager => self.handle_key_board_manager(key),
            InputMode::AddingColumn | InputMode::RenamingColumn => self.handle_key_column_form(key),
        }
    }
}
