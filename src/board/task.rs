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
    #[serde(default)]
    pub url: Option<String>,
}

impl Task {
    pub fn description_len(&self) -> usize {
        self.description.chars().count()
    }

    pub fn is_long_description(&self) -> bool {
        self.description_len() > 200
    }

    #[allow(dead_code)]
    pub fn description_preview(&self, max: usize) -> String {
        if self.description.chars().count() <= max {
            self.description.clone()
        } else {
            let preview: String = self.description.chars().take(max).collect();
            format!("{preview}…")
        }
    }

    #[allow(dead_code)]
    pub fn url_valid(url: &Option<String>) -> bool {
        if let Some(u) = url {
            let t = u.trim();
            if t.is_empty() {
                return true;
            }
            t.starts_with("http://") || t.starts_with("https://")
        } else {
            true
        }
    }

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
