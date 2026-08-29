//! app/handlers/board_manager.rs — Board manager keys

use crate::app::state::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

impl App {
    pub(crate) fn handle_key_board_manager(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('b') | KeyCode::Char('B') => {
                self.close_board_manager();
            }
            KeyCode::Char('a') if key.modifiers.is_empty() => {
                self.close_board_manager();
                self.start_adding_column();
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                self.close_board_manager();
                self.start_renaming_column();
            }
            KeyCode::Char('d') if key.modifiers.is_empty() => self.delete_column(),
            KeyCode::Char('D') if key.modifiers == KeyModifiers::SHIFT => self.delete_column(),
            KeyCode::Char('h') | KeyCode::Left if key.modifiers.is_empty() => self.select_prev_col(),
            KeyCode::Char('l') | KeyCode::Right if key.modifiers.is_empty() => self.select_next_col(),
            KeyCode::Char('H') if key.modifiers == KeyModifiers::SHIFT => self.move_column_left(),
            KeyCode::Char('L') if key.modifiers == KeyModifiers::SHIFT => self.move_column_right(),
            KeyCode::Char('h') if key.modifiers == KeyModifiers::SHIFT => self.move_column_left(),
            KeyCode::Char('l') if key.modifiers == KeyModifiers::SHIFT => self.move_column_right(),
            KeyCode::Char('t') if key.modifiers.is_empty() => {
                self.create_board_from_template();
                self.close_board_manager();
            }
            KeyCode::Char('T') if key.modifiers == KeyModifiers::SHIFT => {
                self.create_board_from_template();
                self.close_board_manager();
            }
            KeyCode::Char('c') if key.modifiers.is_empty() => self.clear_board(),
            KeyCode::Char('C') if key.modifiers == KeyModifiers::SHIFT => self.clear_board(),
            _ => {}
        }
        false
    }
}
