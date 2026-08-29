//! app/ops/navigation.rs — Column/row selection & task movement

use crate::app::state::App;
use crate::storage;

impl App {
    pub fn select_next_row(&mut self) {
        let len = self
            .board
            .columns
            .get(self.selected_col)
            .map(|c| c.tasks.len())
            .unwrap_or(0);
        if len == 0 {
            return;
        }
        if self.selected_row + 1 < len {
            self.selected_row += 1;
        }
    }

    pub fn select_prev_row(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
        }
    }

    pub fn select_next_col(&mut self) {
        if self.selected_col + 1 < self.board.column_count() {
            self.selected_col += 1;
            self.clamp_selection();
        }
    }

    pub fn select_prev_col(&mut self) {
        if self.selected_col > 0 {
            self.selected_col -= 1;
            self.clamp_selection();
        }
    }

    pub fn move_task_to_next_col(&mut self) {
        if self.board.column_count() == 0 {
            return;
        }
        if self.selected_col + 1 >= self.board.column_count() {
            self.status = "Already in last column".to_string();
            return;
        }
        let task = {
            let col = &mut self.board.columns[self.selected_col];
            if col.tasks.is_empty() || self.selected_row >= col.tasks.len() {
                self.status = "No task to move".to_string();
                return;
            }
            col.tasks.remove(self.selected_row)
        };
        let next_col = self.selected_col + 1;
        self.board.columns[next_col].tasks.push(task);

        let new_len = self.board.columns[self.selected_col].tasks.len();
        if new_len == 0 {
            self.selected_row = 0;
        } else if self.selected_row >= new_len {
            self.selected_row = new_len - 1;
        }
        self.selected_col = next_col;
        self.selected_row = self.board.columns[next_col].tasks.len() - 1;
        storage::save_board(&self.board);
        self.status = format!("Moved to '{}'", self.board.columns[next_col].name);
    }

    pub fn move_task_to_prev_col(&mut self) {
        if self.selected_col == 0 {
            self.status = "Already in first column".to_string();
            return;
        }
        let task = {
            let col = &mut self.board.columns[self.selected_col];
            if col.tasks.is_empty() || self.selected_row >= col.tasks.len() {
                self.status = "No task to move".to_string();
                return;
            }
            col.tasks.remove(self.selected_row)
        };
        let prev_col = self.selected_col - 1;
        self.board.columns[prev_col].tasks.push(task);

        let new_len = self.board.columns[self.selected_col].tasks.len();
        if new_len == 0 {
            self.selected_row = 0;
        } else if self.selected_row >= new_len {
            self.selected_row = new_len - 1;
        }
        self.selected_col = prev_col;
        self.selected_row = self.board.columns[prev_col].tasks.len() - 1;
        storage::save_board(&self.board);
        self.status = format!("Moved to '{}'", self.board.columns[prev_col].name);
    }
}
