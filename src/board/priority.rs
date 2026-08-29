//! board/priority.rs — Priority levels for tasks

use serde::{Deserialize, Serialize};

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
