//! board/task.rs — Task model (highly customizable card)

use super::priority::Priority;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    /// Stable numeric id (monotonic via `Board::next_id`).
    pub id: u32,
    /// Short title shown in the column list.
    pub title: String,
    /// Longer description (viewed with `Enter`/`v`).
    #[serde(default)]
    pub description: String,
    /// Optional deadline (YYYY-MM-DD). Stored as string for lenient parsing.
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
