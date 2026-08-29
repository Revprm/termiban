//! board/board.rs — Board container + column management

use super::{column::Column, priority::Priority, task::Task};
use serde::{Deserialize, Serialize};

fn default_board_title() -> String {
    "My Board".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub columns: Vec<Column>,
    pub next_id: u32,
    #[serde(default = "default_board_title")]
    pub title: String,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            title: default_board_title(),
            columns: vec![
                Column {
                    name: "To Do".to_string(),
                    tasks: vec![
                        Task {
                            id: 1,
                            title: "Set up Termiban".to_string(),
                            description: "Customize your board with `B`, add tasks with `n`".to_string(),
                            deadline: None,
                            priority: Priority::High,
                            tags: vec!["setup".into()],
                            created_at: Some(chrono::Local::now().format("%Y-%m-%d").to_string()),
                        },
                        Task {
                            id: 2,
                            title: "Plan your first board".to_string(),
                            description: "Add deadlines, priorities, tags — all persisted".to_string(),
                            deadline: Some(
                                (chrono::Local::now() + chrono::Duration::days(2))
                                    .format("%Y-%m-%d")
                                    .to_string(),
                            ),
                            priority: Priority::Medium,
                            tags: vec!["planning".into()],
                            created_at: Some(chrono::Local::now().format("%Y-%m-%d").to_string()),
                        },
                    ],
                },
                Column {
                    name: "In Progress".to_string(),
                    tasks: vec![Task {
                        id: 3,
                        title: "Learn Ratatui shortcuts".to_string(),
                        description: "Press `?` for help, `Enter` to view task details".to_string(),
                        deadline: None,
                        priority: Priority::Medium,
                        tags: vec![],
                        created_at: Some(chrono::Local::now().format("%Y-%m-%d").to_string()),
                    }],
                },
                Column {
                    name: "Done".to_string(),
                    tasks: vec![Task {
                        id: 4,
                        title: "Install Rust".to_string(),
                        description: "rustup.rs — you are ready!".to_string(),
                        deadline: None,
                        priority: Priority::Low,
                        tags: vec!["done".into()],
                        created_at: Some(chrono::Local::now().format("%Y-%m-%d").to_string()),
                    }],
                },
            ],
            next_id: 5,
        }
    }
}

impl Board {
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    pub fn add_column(&mut self, name: String) {
        let name = name.trim();
        if name.is_empty() {
            return;
        }
        self.columns.push(Column {
            name: name.to_string(),
            tasks: vec![],
        });
    }

    pub fn rename_column(&mut self, idx: usize, new_name: String) {
        let new_name = new_name.trim();
        if new_name.is_empty() || idx >= self.columns.len() {
            return;
        }
        self.columns[idx].name = new_name.to_string();
    }

    pub fn delete_column(&mut self, idx: usize) -> Option<Column> {
        if self.columns.is_empty() {
            return None;
        }
        if idx < self.columns.len() {
            Some(self.columns.remove(idx))
        } else {
            None
        }
    }

    pub fn move_column(&mut self, idx: usize, dir: i32) -> Option<usize> {
        let new_idx = idx as i32 + dir;
        if new_idx < 0 || new_idx >= self.columns.len() as i32 {
            return None;
        }
        let new_idx = new_idx as usize;
        let col = self.columns.remove(idx);
        self.columns.insert(new_idx, col);
        Some(new_idx)
    }

    pub fn reset_to_template(&mut self) {
        *self = Self::default();
    }

    pub fn clear(&mut self) {
        self.columns.clear();
    }
}
