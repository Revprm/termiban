//! app/handlers/normal.rs — Normal mode keys

use crate::app::state::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

impl App {
    pub(crate) fn handle_key_normal(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => return true,
            KeyCode::Char('q') if key.modifiers.is_empty() => return true,
            KeyCode::Char('?') => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return false;
                }
                self.toggle_help();
                return false;
            }
            _ => {}
        }

        // Board manager shortcut (always available, even when empty)
        if key.code == KeyCode::Char('B') && key.modifiers == KeyModifiers::SHIFT {
            self.open_board_manager();
            return false;
        }
        if key.code == KeyCode::Char('b') && key.modifiers.is_empty() {
            self.open_board_manager();
            return false;
        }

        // Empty board handling — show onboarding
        if self.board.is_empty() {
            match key.code {
                KeyCode::Char('a') | KeyCode::Char('n') if key.modifiers.is_empty() => {
                    self.start_adding_column();
                    return false;
                }
                KeyCode::Char('C') if key.modifiers == KeyModifiers::SHIFT => {
                    self.start_adding_column();
                    return false;
                }
                KeyCode::Char('t') if key.modifiers.is_empty() => {
                    self.create_board_from_template();
                    return false;
                }
                KeyCode::Char('T') if key.modifiers == KeyModifiers::SHIFT => {
                    self.create_board_from_template();
                    return false;
                }
                KeyCode::Char('c') if key.modifiers.is_empty() => {
                    self.status = "Board already empty — press 'a' to add first column or 't' for template".to_string();
                    return false;
                }
                _ => {
                    self.status =
                        "Board is empty — press 'a' to add column, 't' for template, 'B' for manager".to_string();
                    return false;
                }
            }
        }

        if key.code == KeyCode::Char('N') && key.modifiers == KeyModifiers::SHIFT {
            self.open_task_form_add();
            return false;
        }
        if key.code == KeyCode::Char('E') && key.modifiers == KeyModifiers::SHIFT {
            self.open_task_form_edit();
            return false;
        }

        if key.code == KeyCode::Char('C') && key.modifiers == KeyModifiers::SHIFT {
            self.start_adding_column();
            return false;
        }
        if key.code == KeyCode::Char('R') && key.modifiers == KeyModifiers::SHIFT {
            self.start_renaming_column();
            return false;
        }

        if (key.code == KeyCode::Enter || key.code == KeyCode::Char('v')) && key.modifiers.is_empty() {
            self.open_detail();
            return false;
        }

        if key.code == KeyCode::Char('p') && key.modifiers.is_empty() {
            self.cycle_priority();
            return false;
        }

        if matches!(key.code, KeyCode::Char('h') | KeyCode::Left) {
            if key.modifiers == KeyModifiers::SHIFT {
                self.move_task_to_prev_col();
            } else if key.modifiers.is_empty() {
                self.select_prev_col();
            } else if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('h') {
                self.move_task_to_prev_col();
            }
            return false;
        }
        if matches!(key.code, KeyCode::Char('l') | KeyCode::Right) {
            if key.modifiers == KeyModifiers::SHIFT {
                self.move_task_to_next_col();
            } else if key.modifiers.is_empty() {
                self.select_next_col();
            } else if key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('l') {
                self.move_task_to_next_col();
            }
            return false;
        }

        match key.code {
            KeyCode::Char('j') | KeyCode::Down if key.modifiers.is_empty() => self.select_next_row(),
            KeyCode::Char('k') | KeyCode::Up if key.modifiers.is_empty() => self.select_prev_row(),
            KeyCode::Char('H') if key.modifiers == KeyModifiers::SHIFT => self.move_task_to_prev_col(),
            KeyCode::Char('L') if key.modifiers == KeyModifiers::SHIFT => self.move_task_to_next_col(),
            KeyCode::Char('n') | KeyCode::Char('a') if key.modifiers.is_empty() => self.start_adding(),
            KeyCode::Char('e') if key.modifiers.is_empty() => self.start_editing(),
            KeyCode::Char('d') | KeyCode::Char('x') if key.modifiers.is_empty() => self.delete_selected(),
            KeyCode::Delete if key.modifiers.is_empty() => self.delete_selected(),
            KeyCode::Char('h') if key.modifiers == KeyModifiers::CONTROL => self.move_task_to_prev_col(),
            KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => self.move_task_to_next_col(),
            _ => {}
        }
        false
    }
}
