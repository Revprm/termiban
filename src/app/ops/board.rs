//! app/ops/board.rs — Board customization (columns)

use crate::app::state::{App, InputMode};
use crate::storage;

impl App {
    pub fn open_board_manager(&mut self) {
        self.input_mode = InputMode::BoardManager;
        self.status = "Board manager — a: add column, r: rename, d: delete, H/L: move column, Esc: close"
            .to_string();
    }

    pub fn close_board_manager(&mut self) {
        self.input_mode = InputMode::Normal;
        self.status = "Closed board manager".to_string();
    }

    pub fn start_adding_column(&mut self) {
        self.input_mode = InputMode::AddingColumn;
        self.input_buffer.clear();
        self.status = "Add column — type name, Enter to confirm, Esc to cancel".to_string();
    }

    pub fn start_renaming_column(&mut self) {
        if self.board.is_empty() {
            self.status = "No column to rename".to_string();
            return;
        }
        let idx = self.selected_col;
        self.renaming_col_idx = Some(idx);
        self.input_mode = InputMode::RenamingColumn;
        self.input_buffer = self.board.columns[idx].name.clone();
        self.status = format!("Rename '{}' — Enter to confirm, Esc to cancel", self.board.columns[idx].name);
    }

    pub fn delete_column(&mut self) {
        if self.board.is_empty() {
            self.status = "No columns to delete".to_string();
            return;
        }
        let idx = self.selected_col;
        if let Some(col) = self.board.delete_column(idx) {
            storage::save_board(&self.board);
            self.clamp_selection();
            if self.board.is_empty() {
                self.status = format!(
                    "Deleted last column '{}' — board is now empty. Press 'a' to add a column or 't' for template",
                    col.name
                );
            } else {
                self.status = format!("Deleted column '{}' ({} tasks removed)", col.name, col.tasks.len());
            }
        }
    }

    pub fn move_column_left(&mut self) {
        let idx = self.selected_col;
        if let Some(new_idx) = self.board.move_column(idx, -1) {
            self.selected_col = new_idx;
            storage::save_board(&self.board);
            self.status = format!("Moved column to position {}", new_idx + 1);
        } else {
            self.status = "Already first column".to_string();
        }
    }

    pub fn move_column_right(&mut self) {
        let idx = self.selected_col;
        if let Some(new_idx) = self.board.move_column(idx, 1) {
            self.selected_col = new_idx;
            storage::save_board(&self.board);
            self.status = format!("Moved column to position {}", new_idx + 1);
        } else {
            self.status = "Already last column".to_string();
        }
    }

    pub fn clear_board(&mut self) {
        self.board.clear();
        self.selected_col = 0;
        self.selected_row = 0;
        storage::save_board(&self.board);
        self.status = "Board cleared — press 'a' to add column or 't' for template".to_string();
    }

    pub fn create_board_from_template(&mut self) {
        self.board.reset_to_template();
        self.selected_col = 0;
        self.selected_row = 0;
        self.input_mode = InputMode::Normal;
        storage::save_board(&self.board);
        self.status = "Created new board from template (To Do / In Progress / Done)".to_string();
    }
}
