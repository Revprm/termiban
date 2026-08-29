//! app/handlers/view.rs — Task detail view keys

use crate::app::state::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

impl App {
    pub(crate) fn handle_key_viewing(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') | KeyCode::Enter => {
                self.close_detail();
            }
            KeyCode::Char('e') if key.modifiers.is_empty() => {
                self.close_detail();
                self.open_task_form_edit();
            }
            KeyCode::Char('E') if key.modifiers == KeyModifiers::SHIFT => {
                self.close_detail();
                self.open_task_form_edit();
            }
            KeyCode::Char('d') if key.modifiers.is_empty() => {
                self.close_detail();
                self.delete_selected();
            }
            KeyCode::Char('p') if key.modifiers.is_empty() => self.cycle_priority(),
            KeyCode::Char('o') if key.modifiers.is_empty() => self.open_selected_url(),
            KeyCode::Char('O') if key.modifiers == KeyModifiers::SHIFT => self.open_selected_url(),
            _ => {}
        }
        false
    }
}
