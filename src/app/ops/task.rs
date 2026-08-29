//! app/ops/task.rs — Task CRUD + rich properties

use crate::app::state::{App, InputMode, TaskForm, TaskFormField};
use crate::board::{Priority, Task};
use crate::storage;
use chrono::NaiveDate;

impl App {
    pub fn start_adding(&mut self) {
        if self.board.is_empty() {
            self.status = "No columns — press 'a' or 'C' to add a column first".to_string();
            self.start_adding_column();
            return;
        }
        self.input_mode = InputMode::Adding;
        self.input_buffer.clear();
        self.editing_id = None;
        self.status = "Quick add — Enter to confirm, Esc to cancel, N for full form".to_string();
    }

    pub fn start_editing(&mut self) {
        if let Some(task) = self.selected_task().cloned() {
            self.input_mode = InputMode::Editing;
            self.input_buffer = task.title.clone();
            self.editing_id = Some(task.id);
            self.status = "Quick edit title — Enter to save, Esc to cancel, E for full edit".to_string();
        } else {
            self.status = "No task to edit".to_string();
        }
    }

    pub fn confirm_input(&mut self) {
        let title = self.input_buffer.trim().to_string();
        if title.is_empty() {
            self.status = "Task title cannot be empty".to_string();
            return;
        }
        match self.input_mode {
            InputMode::Adding => {
                if self.board.is_empty() {
                    self.status = "No columns — add a column first (press 'C')".to_string();
                    self.input_mode = InputMode::Normal;
                    self.input_buffer.clear();
                    return;
                }
                let col_name;
                {
                    let col = &mut self.board.columns[self.selected_col];
                    let task = Task {
                        id: self.board.next_id,
                        title: title.clone(),
                        description: String::new(),
                        deadline: None,
                        priority: Priority::default(),
                        tags: vec![],
                        created_at: Some(chrono::Local::now().format("%Y-%m-%d").to_string()),
                    };
                    self.board.next_id += 1;
                    col.tasks.push(task);
                    self.selected_row = col.tasks.len() - 1;
                    col_name = col.name.clone();
                }
                storage::save_board(&self.board);
                self.status = format!("Added '{title}' to '{col_name}'");
            }
            InputMode::Editing => {
                if let Some(id) = self.editing_id {
                    let mut found = false;
                    for col in &mut self.board.columns {
                        for task in &mut col.tasks {
                            if task.id == id {
                                task.title = title.clone();
                                found = true;
                                break;
                            }
                        }
                    }
                    if found {
                        storage::save_board(&self.board);
                        self.status = format!("Updated to '{title}'");
                    } else {
                        self.status = "Task not found".to_string();
                    }
                }
            }
            InputMode::AddingColumn => {
                let name = title.clone();
                self.board.add_column(name.clone());
                storage::save_board(&self.board);
                self.selected_col = self.board.column_count() - 1;
                self.selected_row = 0;
                self.status = format!("Added column '{name}'");
            }
            InputMode::RenamingColumn => {
                let idx = self.renaming_col_idx.unwrap_or(self.selected_col);
                let old = self.board.columns.get(idx).map(|c| c.name.clone()).unwrap_or_default();
                self.board.rename_column(idx, title.clone());
                storage::save_board(&self.board);
                self.status = format!("Renamed '{old}' → '{title}'");
                self.renaming_col_idx = None;
            }
            _ => {}
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.editing_id = None;
    }

    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
        self.editing_id = None;
        self.renaming_col_idx = None;
        self.status = "Cancelled".to_string();
    }

    pub fn delete_selected(&mut self) {
        if self.board.is_empty() {
            self.status = "No task to delete".to_string();
            return;
        }
        let col = &mut self.board.columns[self.selected_col];
        if col.tasks.is_empty() || self.selected_row >= col.tasks.len() {
            self.status = "No task to delete".to_string();
            return;
        }
        let removed = col.tasks.remove(self.selected_row);
        if self.selected_row >= col.tasks.len() && self.selected_row > 0 {
            self.selected_row -= 1;
        }
        storage::save_board(&self.board);
        self.status = format!("Deleted '{}'", removed.title);
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn cycle_priority(&mut self) {
        if let Some(task) = self.selected_task_mut() {
            let old = task.priority;
            task.priority = task.priority.next();
            let title = task.title.clone();
            let pri = task.priority;
            storage::save_board(&self.board);
            self.status = format!("'{title}': {old} → {pri}");
        } else {
            self.status = "No task to update".to_string();
        }
    }

    pub fn open_detail(&mut self) {
        if self.selected_task().is_some() {
            self.input_mode = InputMode::ViewingTask;
            self.status = "Viewing task — Esc/q to close, E to edit".to_string();
        } else {
            self.status = "No task to view".to_string();
        }
    }

    pub fn close_detail(&mut self) {
        self.input_mode = InputMode::Normal;
        self.status = "Closed detail".to_string();
    }

    pub fn open_task_form_add(&mut self) {
        if self.board.is_empty() {
            self.status = "No columns — press 'a' or 'C' to add a column first".to_string();
            self.start_adding_column();
            return;
        }
        self.task_form = TaskForm {
            title: String::new(),
            description: String::new(),
            deadline: String::new(),
            priority: Priority::Medium,
            tags: String::new(),
            field: TaskFormField::Title,
            editing_id: None,
        };
        self.input_mode = InputMode::TaskForm;
        self.status =
            "Full add — Tab/Shift+Tab to switch field, Enter to save, Esc to cancel".to_string();
    }

    pub fn open_task_form_edit(&mut self) {
        if let Some(task) = self.selected_task().cloned() {
            self.task_form = TaskForm {
                title: task.title.clone(),
                description: task.description.clone(),
                deadline: task.deadline.clone().unwrap_or_default(),
                priority: task.priority,
                tags: task.tags.join(", "),
                field: TaskFormField::Title,
                editing_id: Some(task.id),
            };
            self.input_mode = InputMode::TaskForm;
            self.status =
                "Full edit — Tab to switch field, Enter to save, Esc to cancel".to_string();
        } else {
            self.status = "No task to edit".to_string();
        }
    }

    pub fn save_task_form(&mut self) {
        let title = self.task_form.title.trim().to_string();
        if title.is_empty() {
            self.status = "Title cannot be empty".to_string();
            return;
        }
        let deadline_opt = {
            let d = self.task_form.deadline.trim();
            if d.is_empty() {
                None
            } else {
                match NaiveDate::parse_from_str(d, "%Y-%m-%d") {
                    Ok(_) => Some(d.to_string()),
                    Err(_) => {
                        self.status = "Invalid deadline — use YYYY-MM-DD or leave empty".to_string();
                        return;
                    }
                }
            }
        };

        let tags_vec: Vec<String> = self
            .task_form
            .tags
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if let Some(id) = self.task_form.editing_id {
            let mut found = false;
            for col in &mut self.board.columns {
                for task in &mut col.tasks {
                    if task.id == id {
                        task.title = title.clone();
                        task.description = self.task_form.description.clone();
                        task.deadline = deadline_opt.clone();
                        task.priority = self.task_form.priority;
                        task.tags = tags_vec.clone();
                        found = true;
                        break;
                    }
                }
            }
            if found {
                storage::save_board(&self.board);
                self.status = format!("Updated '{title}'");
            } else {
                self.status = "Task not found".to_string();
            }
        } else {
            if self.board.is_empty() {
                self.status = "No columns — add a column first (press 'C')".to_string();
                return;
            }
            let col_name;
            {
                let col = &mut self.board.columns[self.selected_col];
                let task = Task {
                    id: self.board.next_id,
                    title: title.clone(),
                    description: self.task_form.description.clone(),
                    deadline: deadline_opt.clone(),
                    priority: self.task_form.priority,
                    tags: tags_vec.clone(),
                    created_at: Some(chrono::Local::now().format("%Y-%m-%d").to_string()),
                };
                self.board.next_id += 1;
                col.tasks.push(task);
                self.selected_row = col.tasks.len() - 1;
                col_name = col.name.clone();
            }
            storage::save_board(&self.board);
            self.status = format!("Added '{title}' to '{col_name}'");
        }
        self.input_mode = InputMode::Normal;
        self.task_form = TaskForm::default();
    }

    pub fn cancel_task_form(&mut self) {
        self.input_mode = InputMode::Normal;
        self.task_form = TaskForm::default();
        self.status = "Cancelled form".to_string();
    }

    pub fn task_form_next_field(&mut self) {
        self.task_form.field = self.task_form.field.next();
    }

    pub fn task_form_prev_field(&mut self) {
        self.task_form.field = self.task_form.field.prev();
    }

    pub fn task_form_cycle_priority(&mut self) {
        self.task_form.priority = self.task_form.priority.next();
    }

    pub fn task_form_push_char(&mut self, c: char) {
        match self.task_form.field {
            TaskFormField::Title => self.task_form.title.push(c),
            TaskFormField::Description => self.task_form.description.push(c),
            TaskFormField::Deadline => self.task_form.deadline.push(c),
            TaskFormField::Priority => {} // cycled with 'p'
            TaskFormField::Tags => self.task_form.tags.push(c),
        }
    }

    pub fn task_form_pop_char(&mut self) {
        match self.task_form.field {
            TaskFormField::Title => {
                self.task_form.title.pop();
            }
            TaskFormField::Description => {
                self.task_form.description.pop();
            }
            TaskFormField::Deadline => {
                self.task_form.deadline.pop();
            }
            TaskFormField::Priority => {}
            TaskFormField::Tags => {
                self.task_form.tags.pop();
            }
        }
    }
}
