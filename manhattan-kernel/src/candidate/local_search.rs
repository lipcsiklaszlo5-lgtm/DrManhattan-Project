use crate::structure::KernelStructureGraph;

pub struct CandidateGenerator {
    pub max_depth: usize,
}

impl CandidateGenerator {
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth }
    }

    /// Generate candidate graphs by applying local operators.
    /// Currently: delete each edge one by one as a simple search.
    pub fn generate(&self, base: &KernelStructureGraph, max_candidates: usize) -> Vec<KernelStructureGraph> {
        let mut candidates = Vec::new();
        for edge in &base.edges {
            if candidates.len() >= max_candidates {
                break;
            }
            let mut g = base.clone();
            g.edges.retain(|e| !(e.from == edge.from && e.to == edge.to && e.rel_type == edge.rel_type));
            candidates.push(g);
        }
        // TODO: add more operators (add edge, merge nodes, etc.) with historical success rates
        candidates
    }
}
