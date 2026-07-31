use crate::structure::KernelStructureGraph;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BonsaiHypothesis {
    pub representation: Option<String>,
    pub concepts: Vec<String>,
    pub operators: Vec<String>,
    pub confidence: f32,
}

pub struct HypothesisBus {
    pub pending: Vec<BonsaiHypothesis>,
}

impl HypothesisBus {
    pub fn new() -> Self { Self { pending: Vec::new() } }
    
    pub fn submit(&mut self, hypothesis: BonsaiHypothesis) {
        self.pending.push(hypothesis);
    }
    
    pub fn best(&self) -> Option<&BonsaiHypothesis> {
        self.pending.iter().max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
    }
}
