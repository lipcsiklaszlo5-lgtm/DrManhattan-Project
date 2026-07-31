mod id;
mod types;
pub mod builder;
pub use builder::TaskBuilder;

pub use id::TaskId;
pub use types::{Constraint, ConstraintKind, Priority, TaskType};

use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::structure::KernelStructureGraph;
use crate::adapter::arc::adapter::ArcGrid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub intent: String,
    pub task_type: TaskType,
    pub context: TaskContext,
    pub constraints: Vec<Constraint>,
    pub priority: Priority,
    pub created_at: DateTime<Utc>,
    pub parent_id: Option<TaskId>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub working_dir: Option<PathBuf>,
    pub files: Vec<PathBuf>,
    pub history: Vec<TaskId>,
    pub structure: Option<KernelStructureGraph>,
    pub grid: Option<ArcGrid>,
    pub target_grid: Option<ArcGrid>,
}

impl Task {
    pub fn builder(intent: impl Into<String>) -> TaskBuilder {
        TaskBuilder::new(intent)
    }
    pub fn is_subtask(&self) -> bool {
        self.parent_id.is_some()
    }
    pub fn add_metadata(&mut self, key: impl Into<String>, val: serde_json::Value) {
        self.metadata.insert(key.into(), val);
    }
}

#[cfg(test)]
mod tests;
