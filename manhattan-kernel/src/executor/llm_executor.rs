use crate::task::Task;
use crate::executor::types::{Cost, ExecutorOutput, ExecutorError};
use crate::executor::Executor;

pub struct LlmExecutor {
    pub mock_response: Option<String>,
}

impl LlmExecutor {
    pub fn new() -> Self { Self { mock_response: None } }
    pub fn with_mock_response(mut self, response: String) -> Self { self.mock_response = Some(response); self }
}

impl Executor for LlmExecutor {
    fn executor_id(&self) -> &str { "llm-stub" }
    fn can_handle(&self, _: &Task) -> bool { true }
    fn estimate_cost(&self, _: &Task) -> Cost { Cost { tokens: 100, estimated_usd: 0.01, latency_ms: 200 } }
    fn estimate_confidence(&self, _: &Task) -> f32 { 0.3 }
    fn execute(&self, _: &Task) -> Result<ExecutorOutput, ExecutorError> {
        match &self.mock_response {
            Some(r) => Ok(ExecutorOutput { content: r.clone(), confidence: 0.7, executor_id: self.executor_id().to_string() }),
            None => Err(ExecutorError::LlmError("no mock response configured".into())),
        }
    }
}
