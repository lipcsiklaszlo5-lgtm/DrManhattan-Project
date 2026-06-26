use crate::task::Task;
use crate::executor::types::{Cost, ExecutorOutput, ExecutorError};
pub mod types;

pub trait Executor {
    fn executor_id(&self) -> &str;
    fn can_handle(&self, task: &Task) -> bool;
    fn estimate_cost(&self, task: &Task) -> Cost;
    fn estimate_confidence(&self, task: &Task) -> f32;
    fn execute(&self, task: &Task) -> Result<ExecutorOutput, ExecutorError>;
}

pub struct AlwaysFailExecutor;
impl Executor for AlwaysFailExecutor {
    fn executor_id(&self) -> &str { "always-fail-stub" }
    fn can_handle(&self, _: &Task) -> bool { false }
    fn estimate_cost(&self, _: &Task) -> Cost { Cost { tokens: 0, estimated_usd: 0.0, latency_ms: 0 } }
    fn estimate_confidence(&self, _: &Task) -> f32 { 0.0 }
    fn execute(&self, _: &Task) -> Result<ExecutorOutput, ExecutorError> { Err(ExecutorError::NotSupported) }
}
