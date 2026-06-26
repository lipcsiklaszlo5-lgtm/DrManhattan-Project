use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    pub tokens: u32,
    pub estimated_usd: f32,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorOutput {
    pub content: String,
    pub confidence: f32,
    pub executor_id: String,
}

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ExecutorError {
    #[error("executor does not support this task")] NotSupported,
    #[error("validation failed: {0}")] ValidationFailed(String),
    #[error("timeout")] Timeout,
    #[error("LLM error: {0}")] LlmError(String),
}
