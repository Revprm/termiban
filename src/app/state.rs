//! app/state.rs — Core state types for Termiban

use crate::board::{Board, Priority};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Adding,
    Editing,
    TaskForm,
    ViewingTask,
    BoardManager,
    AddingColumn,
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
    pub fn next(self) -> Self {
        match self {
            Self::Title => Self::Description,
            Self::Description => Self::Deadline,
            Self::Deadline => Self::Priority,
            Self::Priority => Self::Tags,
            Self::Tags => Self::Title,
        }
    }
    pub fn prev(self) -> Self {
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

pub struct App {
    pub board: Board,
    pub selected_col: usize,
    pub selected_row: usize,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub(crate) editing_id: Option<u32>,
    pub task_form: TaskForm,
    pub show_help: bool,
    pub status: String,
    pub(crate) renaming_col_idx: Option<usize>,
}

impl App {
    pub fn new() -> Self {
        let board = crate::storage::load_board();
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
        app.status = format!("Board loaded — {}", crate::storage::data_path().display());
        app
    }

    pub(crate) fn clamp_selection(&mut self) {
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

    pub fn selected_task(&self) -> Option<&crate::board::Task> {
        self.board
            .columns
            .get(self.selected_col)
            .and_then(|c| c.tasks.get(self.selected_row))
    }

    pub(crate) fn selected_task_mut(&mut self) -> Option<&mut crate::board::Task> {
        self.board
            .columns
            .get_mut(self.selected_col)
            .and_then(|c| c.tasks.get_mut(self.selected_row))
    }
}
