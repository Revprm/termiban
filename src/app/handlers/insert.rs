//! app/handlers/insert.rs — Quick add/edit (title only) + column form

use crate::app::state::App;
use crossterm::event::{KeyCode, KeyEvent};

impl App {
    pub(crate) fn handle_key_insert(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => self.confirm_input(),
            KeyCode::Esc => self.cancel_input(),
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => self.input_buffer.push(c),
            _ => {}
        }
        false
    }

    pub(crate) fn handle_key_column_form(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => self.confirm_input(),
            KeyCode::Esc => self.cancel_input(),
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => self.input_buffer.push(c),
            _ => {}
        }
        false
    }
}
