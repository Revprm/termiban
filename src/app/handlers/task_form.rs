//! app/handlers/task_form.rs — Full task form navigation

use crate::app::state::{App, TaskFormField};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

impl App {
    pub(crate) fn handle_key_task_form(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => self.cancel_task_form(),
            KeyCode::Enter if key.modifiers.is_empty() => self.save_task_form(),
            KeyCode::Tab if key.modifiers.is_empty() => self.task_form_next_field(),
            KeyCode::BackTab => self.task_form_prev_field(),
            KeyCode::Tab if key.modifiers == KeyModifiers::SHIFT => self.task_form_prev_field(),
            KeyCode::Char('p') if key.modifiers.is_empty() && self.task_form.field == TaskFormField::Priority => {
                self.task_form_cycle_priority()
            }
            KeyCode::Backspace => self.task_form_pop_char(),
            KeyCode::Char(c) => {
                if self.task_form.field == TaskFormField::Priority {
                    if c == 'p' || c == 'P' {
                        self.task_form_cycle_priority();
                    }
                } else {
                    self.task_form_push_char(c);
                }
            }
            _ => {}
        }
        false
    }
}
