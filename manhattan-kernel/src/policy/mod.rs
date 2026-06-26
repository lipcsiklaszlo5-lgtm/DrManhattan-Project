use crate::task::Task;
use crate::candidate::CandidateGenerator;
use crate::structure::KernelStructureGraph;

#[derive(Debug, Clone)]
pub struct CostModel {
    pub llm_cost_per_call: f32,
}

impl CostModel {
    pub fn estimate_llm_cost(&self, _task: &Task) -> f32 {
        self.llm_cost_per_call
    }
}

pub struct PolicyEngine {
    cost_model: CostModel,
    candidate_gen: CandidateGenerator,
}

impl PolicyEngine {
    pub fn new(cost_model: CostModel, candidate_gen: CandidateGenerator) -> Self {
        Self { cost_model, candidate_gen }
    }

    /// Decision: returns which path to take.
    pub fn decide(&self, task: &Task, has_algorithm: bool, cache_hit: bool) -> &str {
        if has_algorithm {
            return "algorithm";
        }
        if cache_hit {
            return "cache";
        }
        if task.context.structure.is_some() {
            return "local_search";
        }
        if self.cost_model.estimate_llm_cost(task) < 0.02 {
            return "llm";
        }
        "llm"
    }

    pub fn generate_candidates(&self, structure: &KernelStructureGraph, max_candidates: usize) -> Vec<KernelStructureGraph> {
        self.candidate_gen.generate(structure, max_candidates)
    }
}
