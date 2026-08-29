//! app/mod.rs — Application state & handlers

pub mod handlers;
pub mod ops;
pub mod state;

#[allow(unused_imports)]
pub use state::{App, InputMode, TaskForm, TaskFormField};

#[cfg(test)]
mod tests {
    use super::state::{App, InputMode, TaskForm};
    use crate::board::{Board, Column, Priority, Task};
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
                        url: None,
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
        let before = app.board.columns[0].tasks.len();
        app.handle_key(key(KeyCode::Char('d'), KeyModifiers::CONTROL));
        assert_eq!(app.board.columns[0].tasks.len(), before, "Ctrl+D must not delete");
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
        app.handle_key(key(KeyCode::Char('l'), KeyModifiers::empty()));
        assert_eq!(app.selected_col, 1);
        assert_eq!(app.board.columns[0].tasks.len(), 1);
        app.handle_key(key(KeyCode::Char('h'), KeyModifiers::empty()));
        assert_eq!(app.selected_col, 0);
        app.handle_key(key(KeyCode::Char('l'), KeyModifiers::SHIFT));
        assert_eq!(app.board.columns[0].tasks.len(), 0);
        assert_eq!(app.board.columns[1].tasks.len(), 1);
        assert_eq!(app.selected_col, 1);
        app.handle_key(key(KeyCode::Char('h'), KeyModifiers::SHIFT));
        assert_eq!(app.selected_col, 0);
        assert_eq!(app.board.columns[0].tasks.len(), 1);
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
        while !app.board.is_empty() {
            app.selected_col = 0;
            app.delete_column();
        }
        assert!(app.board.is_empty());
        app.handle_key(key(KeyCode::Char('t'), KeyModifiers::empty()));
        assert_eq!(app.board.column_count(), 3);
        assert_eq!(app.board.columns[0].name, "To Do");
        app.clear_board();
        assert!(app.board.is_empty());
        app.handle_key(key(KeyCode::Char('a'), KeyModifiers::empty()));
        assert_eq!(app.input_mode, InputMode::AddingColumn);
        app.input_buffer = "Ideas".into();
        app.confirm_input();
        assert_eq!(app.board.column_count(), 1);
        assert_eq!(app.board.columns[0].name, "Ideas");
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
