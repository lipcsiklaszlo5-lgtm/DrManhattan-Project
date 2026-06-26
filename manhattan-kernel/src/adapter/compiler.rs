use crate::structure::KernelStructureGraph;
use crate::task::Task;
use super::{Algorithm, CostEstimate, DomainAdapter, ValidationError};

pub struct CompilerAdapter;

impl DomainAdapter for CompilerAdapter {
    fn build_structure(&self, task: &Task) -> KernelStructureGraph {
        let mut g = KernelStructureGraph::new();
        if task.intent.contains("E0308") {
            let node = g.add_node("err1", "compiler_error");
            node.attributes.insert("code".into(), "E0308".into());
            g.add_edge("err1", "main_fn", "occurs_in");
        } else if task.intent.contains("lifetime") {
            g.add_node("err2", "lifetime_error");
        }
        g
    }

    fn validate(&self, _structure: &KernelStructureGraph, candidate: &str) -> Result<(), ValidationError> {
        if candidate.contains("correct fix") {
            Ok(())
        } else {
            Err(ValidationError::Failed("invalid candidate".into()))
        }
    }

    fn available_algorithms(&self) -> Vec<Algorithm> {
        vec![
            Algorithm {
                name: "cargo_fmt".into(),
                description: "format code".into(),
                cost: CostEstimate { latency_ms: 50, memory_bytes: 1024 },
            },
            Algorithm {
                name: "cargo_check".into(),
                description: "type check".into(),
                cost: CostEstimate { latency_ms: 200, memory_bytes: 4096 },
            },
        ]
    }
}
