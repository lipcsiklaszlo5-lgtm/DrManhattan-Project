use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskType {
    CodeGeneration,
    CodeValidation,
    Explanation,
    Search,
    Transformation,
    Classification,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConstraintKind {
    MaxTokens(u32),
    MaxLatencyMs(u64),
    MaxCostUsd(f32),
    RequiredExecutor(String),
    ForbiddenExecutor(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constraint {
    pub kind: ConstraintKind,
}
