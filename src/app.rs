//! app.rs — Application state & business logic
//!
//! Highly customizable:
//! * Rich `Task` (deadline, priority, tags, description)
//! * Dynamic `Board` (add/rename/delete/reorder columns)
//! * Full task form (Tab between fields) + quick add
//! * Detail view, priority cycling, board manager

use crate::{
    board::{Board, Priority, Task},
    storage,
};
use chrono::NaiveDate;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// ---------------------------------------------------------------------------
// Input mode & forms
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    /// Quick add: title only
    Adding,
    /// Quick edit: title only
    Editing,
    /// Full task form (add or edit)
    TaskForm,
    /// Viewing detail of a task (read-only)
    ViewingTask,
    /// Board manager overlay
    BoardManager,
    /// Adding a new column (buffer = name)
    AddingColumn,
    /// Renaming existing column
    RenamingColumn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskFormField {
    Title,
    Description,
    Deadline,
    Priority,
    Tags,
}

impl TaskFormField {
    fn next(self) -> Self {
        match self {
            Self::Title => Self::Description,
            Self::Description => Self::Deadline,
            Self::Deadline => Self::Priority,
            Self::Priority => Self::Tags,
            Self::Tags => Self::Title,
        }
    }
    fn prev(self) -> Self {
        match self {
            Self::Title => Self::Tags,
            Self::Description => Self::Title,
            Self::Deadline => Self::Description,
            Self::Priority => Self::Deadline,
            Self::Tags => Self::Priority,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Description => "Description",
            Self::Deadline => "Deadline (YYYY-MM-DD)",
            Self::Priority => "Priority",
            Self::Tags => "Tags (comma-separated)",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskForm {
    pub title: String,
    pub description: String,
    pub deadline: String,
    pub priority: Priority,
    pub tags: String,
    pub field: TaskFormField,
    /// None => add, Some(id) => edit
    pub editing_id: Option<u32>,
}

impl Default for TaskForm {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            deadline: String::new(),
            priority: Priority::Medium,
            tags: String::new(),
            field: TaskFormField::Title,
            editing_id: None,
        }
    }
}

// ---------------------------------------------------------------------------
// App — central state
// ---------------------------------------------------------------------------

pub struct App {
    /// Persisted Kanban board.
    pub board: Board,
    /// Index of the highlighted column.
    pub selected_col: usize,
    /// Index of the highlighted task within `selected_col`.
    pub selected_row: usize,
    /// Current keyboard mode.
    pub input_mode: InputMode,
    /// Buffer for quick add/edit & column name
    pub input_buffer: String,
    /// Task id being edited (quick edit)
    editing_id: Option<u32>,
    /// Full task form (when in TaskForm mode)
    pub task_form: TaskForm,
    /// Whether help overlay is visible.
    pub show_help: bool,
    /// One-line status / hint shown in the footer.
    pub status: String,
    /// For RenamingColumn, which column idx is being renamed
    renaming_col_idx: Option<usize>,
}

impl App {
    /// Create app, loading board from disk.
    pub fn new() -> Self {
        let board = storage::load_board();
        let mut app = Self {
            board,
            selected_col: 0,
            selected_row: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            editing_id: None,
            task_form: TaskForm::default(),
            show_help: false,
            status: String::new(),
            renaming_col_idx: None,
        };
        app.clamp_selection();
        app.status = format!("Board loaded — {}", storage::data_path().display());
        app
    }

    // -- selection helpers --------------------------------------------------

    fn clamp_selection(&mut self) {
        if self.board.is_empty() {
            self.selected_col = 0;
            self.selected_row = 0;
            return;
        }
        self.selected_col = self.selected_col.min(self.board.columns.len() - 1);
        let len = self.board.columns[self.selected_col].tasks.len();
        if len == 0 {
            self.selected_row = 0;
        } else {
            self.selected_row = self.selected_row.min(len - 1);
        }
    }

    /// Currently highlighted task, if any.
    pub fn selected_task(&self) -> Option<&Task> {
        self.board
            .columns
            .get(self.selected_col)
            .and_then(|c| c.tasks.get(self.selected_row))
    }

    fn selected_task_mut(&mut self) -> Option<&mut Task> {
        self.board
            .columns
            .get_mut(self.selected_col)
            .and_then(|c| c.tasks.get_mut(self.selected_row))
    }

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

    // -- task movement ------------------------------------------------------

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

    // -- quick CRUD (title only) -------------------------------------------

    pub fn start_adding(&mut self) {
        if self.board.is_empty() {
            self.status = "No columns — press 'a' or 'C' to add a column first".to_string();
            // Directly open column creation for empty board
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

    // -- rich task features -------------------------------------------------

    /// Cycle priority of selected task: Low→Medium→High→Urgent
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

    /// Open detail view for selected task
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

    /// Open full form for adding (empty)
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

    /// Open full form for editing selected task
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

    /// Save the full task form (validate deadline)
    pub fn save_task_form(&mut self) {
        let title = self.task_form.title.trim().to_string();
        if title.is_empty() {
            self.status = "Title cannot be empty".to_string();
            return;
        }
        // Validate deadline if present
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
            // Edit existing
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
            // Add new
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

    /// Handle typing for the focused field in the task form
    pub fn task_form_push_char(&mut self, c: char) {
        match self.task_form.field {
            TaskFormField::Title => self.task_form.title.push(c),
            TaskFormField::Description => self.task_form.description.push(c),
            TaskFormField::Deadline => self.task_form.deadline.push(c),
            TaskFormField::Priority => {} // priority is cycled with 'p', not typed
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

    // -- board customization ------------------------------------------------

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

    // -- central key dispatcher (keeps `main.rs` slim) ---------------------

    /// Handle a key event. Returns `true` if the app should quit.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Help overlay eats everything: any key closes it.
        if self.show_help {
            match key.code {
                KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Esc => {
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

    fn handle_key_normal(&mut self, key: KeyEvent) -> bool {
        // Only plain keys (no modifiers) trigger most actions.
        // This prevents Ctrl+D (EOF), Ctrl+C etc. from being misinterpreted as 'd', 'c', ...
        // For '?' we allow SHIFT because '?' is Shift+'/' on US layouts.
        match key.code {
            KeyCode::Esc => return true,
            KeyCode::Char('q') if key.modifiers.is_empty() => return true,
            KeyCode::Char('?') => {
                // '?' may come with SHIFT; allow it regardless of Shift state, but not Ctrl
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
            // also allow plain 'b' for board manager (convenient)
            self.open_board_manager();
            return false;
        }

        // --- Empty board handling — show onboarding ---
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
                    // clear already empty
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

        // Full form shortcuts: N (add), E (edit) with Shift
        if key.code == KeyCode::Char('N') && key.modifiers == KeyModifiers::SHIFT {
            self.open_task_form_add();
            return false;
        }
        if key.code == KeyCode::Char('E') && key.modifiers == KeyModifiers::SHIFT {
            self.open_task_form_edit();
            return false;
        }

        // Quick column ops (no board manager needed)
        if key.code == KeyCode::Char('C') && key.modifiers == KeyModifiers::SHIFT {
            self.start_adding_column();
            return false;
        }
        if key.code == KeyCode::Char('R') && key.modifiers == KeyModifiers::SHIFT {
            self.start_renaming_column();
            return false;
        }

        // Detail view
        if (key.code == KeyCode::Enter || key.code == KeyCode::Char('v')) && key.modifiers.is_empty() {
            self.open_detail();
            return false;
        }

        // Priority cycle quick
        if key.code == KeyCode::Char('p') && key.modifiers.is_empty() {
            self.cycle_priority();
            return false;
        }

        // Navigation: h/l with SHIFT moves task, otherwise changes column.
        // Arrow keys behave the same (Shift+Arrow = move).
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

        // Other navigation / CRUD — only when no modifiers (strict)
        match key.code {
            KeyCode::Char('j') | KeyCode::Down if key.modifiers.is_empty() => {
                self.select_next_row()
            }
            KeyCode::Char('k') | KeyCode::Up if key.modifiers.is_empty() => {
                self.select_prev_row()
            }

            // Explicit uppercase H/L (often reported as Shift+ h/l)
            KeyCode::Char('H') if key.modifiers == KeyModifiers::SHIFT => {
                self.move_task_to_prev_col()
            }
            KeyCode::Char('L') if key.modifiers == KeyModifiers::SHIFT => {
                self.move_task_to_next_col()
            }

            // CRUD — strict: no modifiers
            KeyCode::Char('n') | KeyCode::Char('a') if key.modifiers.is_empty() => {
                self.start_adding()
            }
            KeyCode::Char('e') if key.modifiers.is_empty() => self.start_editing(),
            KeyCode::Char('d') | KeyCode::Char('x') if key.modifiers.is_empty() => {
                self.delete_selected()
            }
            KeyCode::Delete if key.modifiers.is_empty() => self.delete_selected(),

            // Ctrl+h/l alternative move binding (already handled above, but keep for completeness)
            KeyCode::Char('h') if key.modifiers == KeyModifiers::CONTROL => {
                self.move_task_to_prev_col()
            }
            KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                self.move_task_to_next_col()
            }
            _ => {}
        }
        false
    }

    fn handle_key_insert(&mut self, key: KeyEvent) -> bool {
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

    fn handle_key_task_form(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.cancel_task_form();
            }
            KeyCode::Enter if key.modifiers.is_empty() => {
                // Enter saves, but if we're in Description field, maybe allow newline?
                // For now, Enter always saves; Shift+Enter could be newline but we keep simple.
                self.save_task_form();
            }
            KeyCode::Tab if key.modifiers.is_empty() => {
                self.task_form_next_field();
            }
            KeyCode::BackTab => {
                self.task_form_prev_field();
            }
            KeyCode::Tab if key.modifiers == KeyModifiers::SHIFT => {
                self.task_form_prev_field();
            }
            KeyCode::Char('p') if key.modifiers.is_empty() && self.task_form.field == TaskFormField::Priority => {
                self.task_form_cycle_priority();
            }
            KeyCode::Backspace => {
                self.task_form_pop_char();
            }
            KeyCode::Char(c) => {
                // For Priority field, typing cycles priority instead
                if self.task_form.field == TaskFormField::Priority {
                    // allow p to cycle, otherwise ignore typing
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

    fn handle_key_viewing(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') | KeyCode::Enter => {
                self.close_detail();
            }
            KeyCode::Char('e') if key.modifiers.is_empty() => {
                // Edit from detail view
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
            KeyCode::Char('p') if key.modifiers.is_empty() => {
                self.cycle_priority();
            }
            _ => {}
        }
        false
    }

    fn handle_key_board_manager(&mut self, key: KeyEvent) -> bool {
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
            KeyCode::Char('d') if key.modifiers.is_empty() => {
                self.delete_column();
            }
            KeyCode::Char('D') if key.modifiers == KeyModifiers::SHIFT => {
                self.delete_column();
            }
            KeyCode::Char('h') | KeyCode::Left if key.modifiers.is_empty() => {
                self.select_prev_col();
            }
            KeyCode::Char('l') | KeyCode::Right if key.modifiers.is_empty() => {
                self.select_next_col();
            }
            KeyCode::Char('H') if key.modifiers == KeyModifiers::SHIFT => {
                self.move_column_left();
            }
            KeyCode::Char('L') if key.modifiers == KeyModifiers::SHIFT => {
                self.move_column_right();
            }
            KeyCode::Char('h') if key.modifiers == KeyModifiers::SHIFT => {
                self.move_column_left();
            }
            KeyCode::Char('l') if key.modifiers == KeyModifiers::SHIFT => {
                self.move_column_right();
            }
            KeyCode::Char('t') if key.modifiers.is_empty() => {
                // Create from template — useful when board is empty
                self.create_board_from_template();
                self.close_board_manager();
            }
            KeyCode::Char('T') if key.modifiers == KeyModifiers::SHIFT => {
                self.create_board_from_template();
                self.close_board_manager();
            }
            KeyCode::Char('c') if key.modifiers.is_empty() => {
                // Clear board (delete all) — start from scratch
                self.clear_board();
            }
            KeyCode::Char('C') if key.modifiers == KeyModifiers::SHIFT => {
                self.clear_board();
            }
            _ => {}
        }
        false
    }

    fn handle_key_column_form(&mut self, key: KeyEvent) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Column, Task};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: mods,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn sample_board() -> Board {
        Board {
            title: "Test".into(),
            columns: vec![
                Column {
                    name: "To Do".into(),
                    tasks: vec![Task {
                        id: 1,
                        title: "A".into(),
                        description: "".into(),
                        deadline: None,
                        priority: Priority::Medium,
                        tags: vec![],
                        created_at: None,
                    }],
                },
                Column {
                    name: "Doing".into(),
                    tasks: vec![],
                },
                Column {
                    name: "Done".into(),
                    tasks: vec![],
                },
            ],
            next_id: 2,
        }
    }

    #[test]
    fn plain_d_deletes_ctrl_d_does_not() {
        let dir = std::env::temp_dir().join(format!("termiban_app_test_{}", std::process::id()));
        let path = dir.join("board.json");
        unsafe { std::env::set_var("TERMIBAN_DATA_PATH", &path) };

        // App with one task
        let mut app = App {
            board: sample_board(),
            selected_col: 0,
            selected_row: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            editing_id: None,
            task_form: TaskForm::default(),
            show_help: false,
            status: String::new(),
            renaming_col_idx: None,
        };

        // Ctrl+D should NOT delete
        let before = app.board.columns[0].tasks.len();
        app.handle_key(key(KeyCode::Char('d'), KeyModifiers::CONTROL));
        assert_eq!(app.board.columns[0].tasks.len(), before, "Ctrl+D must not delete");

        // Plain d SHOULD delete
        app.handle_key(key(KeyCode::Char('d'), KeyModifiers::empty()));
        assert_eq!(app.board.columns[0].tasks.len(), 0, "plain d must delete");

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
        unsafe { std::env::remove_var("TERMIBAN_DATA_PATH") };
    }

    #[test]
    fn navigation_and_move_respects_modifiers() {
        let dir = std::env::temp_dir().join(format!("termiban_nav_test_{}", std::process::id()));
        let path = dir.join("board.json");
        unsafe { std::env::set_var("TERMIBAN_DATA_PATH", &path) };

        let mut app = App {
            board: sample_board(),
            selected_col: 0,
            selected_row: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            editing_id: None,
            task_form: TaskForm::default(),
            show_help: false,
            status: String::new(),
            renaming_col_idx: None,
        };

        // Plain l moves column (no task move)
        app.handle_key(key(KeyCode::Char('l'), KeyModifiers::empty()));
        assert_eq!(app.selected_col, 1);
        assert_eq!(app.board.columns[0].tasks.len(), 1);
        // Back to To Do
        app.handle_key(key(KeyCode::Char('h'), KeyModifiers::empty()));
        assert_eq!(app.selected_col, 0);

        // Shift+l moves task from To Do to Doing
        app.handle_key(key(KeyCode::Char('l'), KeyModifiers::SHIFT));
        assert_eq!(app.board.columns[0].tasks.len(), 0);
        assert_eq!(app.board.columns[1].tasks.len(), 1);
        assert_eq!(app.selected_col, 1);

        // Shift+h moves it back
        app.handle_key(key(KeyCode::Char('h'), KeyModifiers::SHIFT));
        assert_eq!(app.selected_col, 0);
        assert_eq!(app.board.columns[0].tasks.len(), 1);

        // Ctrl+h also moves (alternative binding)
        app.handle_key(key(KeyCode::Char('l'), KeyModifiers::SHIFT));
        assert_eq!(app.selected_col, 1);
        app.handle_key(key(KeyCode::Char('h'), KeyModifiers::CONTROL));
        assert_eq!(app.selected_col, 0);
        assert_eq!(app.board.columns[0].tasks.len(), 1);

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
        unsafe { std::env::remove_var("TERMIBAN_DATA_PATH") };
    }

    #[test]
    fn task_form_validation() {
        let dir = std::env::temp_dir().join(format!("termiban_form_test_{}", std::process::id()));
        let path = dir.join("board.json");
        unsafe { std::env::set_var("TERMIBAN_DATA_PATH", &path) };

        let mut app = App {
            board: sample_board(),
            selected_col: 0,
            selected_row: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            editing_id: None,
            task_form: TaskForm::default(),
            show_help: false,
            status: String::new(),
            renaming_col_idx: None,
        };

        app.open_task_form_add();
        app.task_form.title = "New Task".into();
        app.task_form.deadline = "invalid-date".into();
        app.save_task_form();
        assert!(app.status.contains("Invalid deadline"));
        // Should stay in form
        assert_eq!(app.input_mode, InputMode::TaskForm);

        app.task_form.deadline = "2026-12-31".into();
        app.task_form.tags = "work, urgent".into();
        app.save_task_form();
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.board.columns[0].tasks.len(), 2);
        let added = app.board.columns[0].tasks.last().unwrap();
        assert_eq!(added.tags, vec!["work", "urgent"]);
        assert_eq!(added.deadline.as_deref(), Some("2026-12-31"));

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
        unsafe { std::env::remove_var("TERMIBAN_DATA_PATH") };
    }

    #[test]
    fn board_column_ops_via_app() {
        let dir = std::env::temp_dir().join(format!("termiban_board_test_{}", std::process::id()));
        let path = dir.join("board.json");
        unsafe { std::env::set_var("TERMIBAN_DATA_PATH", &path) };

        let mut app = App {
            board: sample_board(),
            selected_col: 0,
            selected_row: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            editing_id: None,
            task_form: TaskForm::default(),
            show_help: false,
            status: String::new(),
            renaming_col_idx: None,
        };

        let n = app.board.column_count();
        app.board.add_column("Review".into());
        assert_eq!(app.board.column_count(), n + 1);
        app.selected_col = n;
        app.start_renaming_column();
        app.input_buffer = "QA".into();
        app.confirm_input();
        assert_eq!(app.board.columns[n].name, "QA");
        app.move_column_left();
        assert_eq!(app.board.columns[n - 1].name, "QA");

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
        unsafe { std::env::remove_var("TERMIBAN_DATA_PATH") };
    }

    #[test]
    fn empty_board_can_be_created_from_scratch() {
        let dir = std::env::temp_dir().join(format!("termiban_empty_test_{}", std::process::id()));
        let path = dir.join("board.json");
        unsafe { std::env::set_var("TERMIBAN_DATA_PATH", &path) };

        let mut app = App {
            board: sample_board(),
            selected_col: 0,
            selected_row: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            editing_id: None,
            task_form: TaskForm::default(),
            show_help: false,
            status: String::new(),
            renaming_col_idx: None,
        };

        // Delete all columns — should be allowed now
        while !app.board.is_empty() {
            app.selected_col = 0;
            app.delete_column();
        }
        assert!(app.board.is_empty());
        assert!(app.board.column_count() == 0);

        // Welcome: press 't' to create from template
        app.handle_key(key(KeyCode::Char('t'), KeyModifiers::empty()));
        assert_eq!(app.board.column_count(), 3);
        assert_eq!(app.board.columns[0].name, "To Do");

        // Clear again
        app.clear_board();
        assert!(app.board.is_empty());

        // Press 'a' to add first column from scratch
        app.handle_key(key(KeyCode::Char('a'), KeyModifiers::empty()));
        // Should be in AddingColumn mode
        assert_eq!(app.input_mode, InputMode::AddingColumn);
        app.input_buffer = "Ideas".into();
        app.confirm_input();
        assert_eq!(app.board.column_count(), 1);
        assert_eq!(app.board.columns[0].name, "Ideas");

        // Can now add task to that column (full form)
        app.open_task_form_add();
        assert_eq!(app.input_mode, InputMode::TaskForm);
        app.task_form.title = "First idea".into();
        app.task_form.description = "My custom board works!".into();
        app.save_task_form();
        assert_eq!(app.board.columns[0].tasks.len(), 1);
        assert_eq!(app.board.columns[0].tasks[0].title, "First idea");

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
        unsafe { std::env::remove_var("TERMIBAN_DATA_PATH") };
    }
}
