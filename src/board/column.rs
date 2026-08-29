//! board/column.rs — Column model

use super::task::Task;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// Display name rendered as block title.
    pub name: String,
    /// Tasks in this column, top-to-bottom order.
    pub tasks: Vec<Task>,
}
