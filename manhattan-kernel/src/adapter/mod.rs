use crate::structure::KernelStructureGraph;
use crate::task::Task;

pub mod compiler;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("validation failed: {0}")]
    Failed(String),
}

pub struct Algorithm {
    pub name: String,
    pub description: String,
    pub cost: CostEstimate,
}

#[derive(Debug, Clone)]
pub struct CostEstimate {
    pub latency_ms: u64,
    pub memory_bytes: u64,
}

pub trait DomainAdapter {
    fn build_structure(&self, task: &Task) -> KernelStructureGraph;
    fn validate(&self, structure: &KernelStructureGraph, candidate: &str) -> Result<(), ValidationError>;
    fn available_algorithms(&self) -> Vec<Algorithm>;
}
#[cfg(test)] mod tests;
