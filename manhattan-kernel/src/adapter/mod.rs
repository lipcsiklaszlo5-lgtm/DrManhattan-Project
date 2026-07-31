use crate::structure::KernelStructureGraph;
use crate::task::Task;

pub mod compiler;
pub mod arc;

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
    fn graph_to_code(&self, graph: &KernelStructureGraph, _original_code: &str) -> String {
        format!("{:?}", graph)
    }
}

#[cfg(test)]
mod tests;
