//! board/column.rs — Column model

use super::task::Task;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub tasks: Vec<Task>,
}
