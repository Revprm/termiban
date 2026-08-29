//! board/task.rs — Task model (highly customizable card)

use super::priority::Priority;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub id: u32,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub deadline: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

impl Task {
    pub fn deadline_date(&self) -> Option<NaiveDate> {
        self.deadline
            .as_deref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    }

    pub fn is_overdue(&self) -> bool {
        if let Some(d) = self.deadline_date() {
            let today = chrono::Local::now().date_naive();
            d < today
        } else {
            false
        }
    }

    pub fn is_due_today(&self) -> bool {
        if let Some(d) = self.deadline_date() {
            let today = chrono::Local::now().date_naive();
            d == today
        } else {
            false
        }
    }

    pub fn tags_display(&self) -> String {
        self.tags.join(", ")
    }
}
