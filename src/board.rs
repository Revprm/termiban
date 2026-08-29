//! board.rs — Pure data model for Termiban
//!
//! Highly customizable: tasks carry deadline, priority, tags, description;
//! board supports dynamic columns. All new fields use `#[serde(default)]`
//! for painless migration from old `board.json`.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Priority — visual + sortable
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Medium
    }
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Urgent => "Urgent",
        };
        write!(f, "{s}")
    }
}

impl Priority {
    /// Cycle Low → Medium → High → Urgent → Low
    pub fn next(self) -> Self {
        match self {
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Urgent,
            Self::Urgent => Self::Low,
        }
    }

    /// One-char icon for compact list view
    pub fn icon(self) -> &'static str {
        match self {
            Self::Low => "·",
            Self::Medium => "●",
            Self::High => "▲",
            Self::Urgent => "‼",
        }
    }
}

// ---------------------------------------------------------------------------
// Task — highly customizable card
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    /// Stable numeric id (monotonic via `Board::next_id`).
    pub id: u32,
    /// Short title shown in the column list.
    pub title: String,
    /// Longer markdown-ish description (viewed with `Enter`/`v`).
    #[serde(default)]
    pub description: String,
    /// Optional deadline (YYYY-MM-DD). Stored as string for lenient parsing,
    /// but helpers provide `NaiveDate` conversion.
    #[serde(default)]
    pub deadline: Option<String>,
    /// Priority drives color/icon.
    #[serde(default)]
    pub priority: Priority,
    /// Free-form tags, e.g. ["work","bug"] — comma-separated in the form.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Creation date (YYYY-MM-DD), auto-set on creation.
    #[serde(default)]
    pub created_at: Option<String>,
}

impl Task {
    /// Parse `deadline` as `NaiveDate` if present and valid.
    pub fn deadline_date(&self) -> Option<NaiveDate> {
        self.deadline
            .as_deref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    }

    /// Is deadline overdue (today > deadline)?
    pub fn is_overdue(&self) -> bool {
        if let Some(d) = self.deadline_date() {
            let today = chrono::Local::now().date_naive();
            d < today
        } else {
            false
        }
    }

    /// Is deadline today?
    pub fn is_due_today(&self) -> bool {
        if let Some(d) = self.deadline_date() {
            let today = chrono::Local::now().date_naive();
            d == today
        } else {
            false
        }
    }

    /// Compact tags as "a, b, c"
    pub fn tags_display(&self) -> String {
        self.tags.join(", ")
    }
}

// ---------------------------------------------------------------------------
// Column
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// Display name rendered as block title.
    pub name: String,
    /// Tasks in this column, top-to-bottom order.
    pub tasks: Vec<Task>,
}

// ---------------------------------------------------------------------------
// Board — dynamic columns, fully customizable
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// Ordered columns — user-customizable via `B` menu.
    pub columns: Vec<Column>,
    /// Next id to assign; bumped on every `Task` creation.
    pub next_id: u32,
    /// Board title (shown in footer). Customizable.
    #[serde(default = "default_board_title")]
    pub title: String,
}

fn default_board_title() -> String {
    "My Board".to_string()
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
    /// Number of columns (generic helper for layout & navigation).
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Convenience: is the board empty (no columns)?
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    // -- column management (highly customizable board) ---------------------

    /// Add a new empty column at the end.
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

    /// Rename column at `idx`.
    pub fn rename_column(&mut self, idx: usize, new_name: String) {
        let new_name = new_name.trim();
        if new_name.is_empty() || idx >= self.columns.len() {
            return;
        }
        self.columns[idx].name = new_name.to_string();
    }

    /// Delete column at `idx`. Returns the removed column.
    /// Now allows deleting the last column — board may become empty.
    /// Caller should handle empty-board onboarding (welcome screen).
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

    /// Create a fresh board from the default template (for onboarding).
    pub fn reset_to_template(&mut self) {
        *self = Self::default();
    }

    /// Clear all columns — start completely from scratch.
    pub fn clear(&mut self) {
        self.columns.clear();
    }

    /// Move column left/right (reorder).
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_three_columns() {
        let b = Board::default();
        assert_eq!(b.column_count(), 3);
        assert_eq!(b.next_id, 5);
    }

    #[test]
    fn task_serde_roundtrip_with_new_fields() {
        let task = Task {
            id: 42,
            title: "Test".into(),
            description: "desc".into(),
            deadline: Some("2026-12-31".into()),
            priority: Priority::Urgent,
            tags: vec!["work".into(), "bug".into()],
            created_at: Some("2026-01-01".into()),
        };
        let json = serde_json::to_string(&task).unwrap();
        let back: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(task, back);
    }

    #[test]
    fn priority_cycle() {
        assert_eq!(Priority::Low.next(), Priority::Medium);
        assert_eq!(Priority::Urgent.next(), Priority::Low);
    }

    #[test]
    fn board_column_ops() {
        let mut b = Board::default();
        let n = b.column_count();
        b.add_column("Review".into());
        assert_eq!(b.column_count(), n + 1);
        b.rename_column(n, "QA".into());
        assert_eq!(b.columns[n].name, "QA");
        let moved = b.move_column(n, -1).unwrap();
        assert_eq!(b.columns[moved].name, "QA");
        assert!(b.delete_column(moved).is_some());
        assert_eq!(b.column_count(), n);
    }

    #[test]
    fn board_can_be_empty_and_rebuilt() {
        let mut b = Board::default();
        // Delete all columns — previously blocked, now allowed
        while !b.is_empty() {
            assert!(b.delete_column(0).is_some());
        }
        assert_eq!(b.column_count(), 0);
        assert!(b.is_empty());
        // Start from scratch: add first column
        b.add_column("Ideas".into());
        assert_eq!(b.column_count(), 1);
        assert_eq!(b.columns[0].name, "Ideas");
        // Create from template
        b.reset_to_template();
        assert_eq!(b.column_count(), 3);
        // Clear again
        b.clear();
        assert!(b.is_empty());
    }

    #[test]
    fn deadline_overdue_logic() {
        let t = Task {
            id: 1,
            title: "x".into(),
            description: "".into(),
            deadline: Some("2000-01-01".into()),
            priority: Priority::Medium,
            tags: vec![],
            created_at: None,
        };
        assert!(t.is_overdue());
        let t2 = Task {
            deadline: Some("2099-01-01".into()),
            ..t.clone()
        };
        assert!(!t2.is_overdue());
    }

    #[test]
    fn migration_old_json_still_loads() {
        // Old board without new fields should still parse via #[serde(default)]
        let old = r#"{"columns":[{"name":"To Do","tasks":[{"id":1,"title":"Old","description":"hi"}]}],"next_id":2,"title":"Old Board"}"#;
        let b: Board = serde_json::from_str(old).unwrap();
        assert_eq!(b.columns[0].tasks[0].priority, Priority::Medium);
        assert!(b.columns[0].tasks[0].deadline.is_none());
        assert!(b.columns[0].tasks[0].tags.is_empty());
    }
}
