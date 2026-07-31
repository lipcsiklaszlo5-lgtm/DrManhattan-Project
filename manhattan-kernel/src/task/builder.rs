use std::collections::HashMap;
use std::path::PathBuf;
use chrono::Utc;
use serde_json::Value;
use crate::task::{Constraint, Priority, Task, TaskContext, TaskId, TaskType};
use crate::structure::KernelStructureGraph;
use crate::adapter::arc::adapter::ArcGrid;

pub struct TaskBuilder {
    intent: String,
    task_type: TaskType,
    priority: Priority,
    constraints: Vec<Constraint>,
    working_dir: Option<PathBuf>,
    files: Vec<PathBuf>,
    parent_id: Option<TaskId>,
    metadata: HashMap<String, Value>,
    structure: Option<KernelStructureGraph>,
    grid: Option<ArcGrid>,
    target_grid: Option<ArcGrid>,
}

impl TaskBuilder {
    pub fn new(intent: impl Into<String>) -> Self {
        Self {
            intent: intent.into(), task_type: TaskType::Unknown, priority: Priority::Normal,
            constraints: Vec::new(), working_dir: None, files: Vec::new(),
            parent_id: None, metadata: HashMap::new(), structure: None, grid: None,
            target_grid: None,
        }
    }
    pub fn task_type(mut self, t: TaskType) -> Self { self.task_type = t; self }
    pub fn priority(mut self, p: Priority) -> Self { self.priority = p; self }
    pub fn constraint(mut self, c: Constraint) -> Self { self.constraints.push(c); self }
    pub fn working_dir(mut self, dir: PathBuf) -> Self { self.working_dir = Some(dir); self }
    pub fn file(mut self, path: PathBuf) -> Self { self.files.push(path); self }
    pub fn parent(mut self, id: TaskId) -> Self { self.parent_id = Some(id); self }
    pub fn metadata(mut self, key: impl Into<String>, val: Value) -> Self { self.metadata.insert(key.into(), val); self }
    pub fn structure(mut self, g: KernelStructureGraph) -> Self { self.structure = Some(g); self }
    pub fn grid(mut self, g: ArcGrid) -> Self { self.grid = Some(g); self }
    pub fn target_grid(mut self, g: ArcGrid) -> Self { self.target_grid = Some(g); self }

    pub fn build(self) -> Task {
        Task {
            id: TaskId::new(), intent: self.intent, task_type: self.task_type,
            context: TaskContext {
                working_dir: self.working_dir, files: self.files, history: Vec::new(),
                structure: self.structure, grid: self.grid, target_grid: self.target_grid,
            },
            constraints: self.constraints, priority: self.priority,
            created_at: Utc::now(), parent_id: self.parent_id, metadata: self.metadata,
        }
    }
}
