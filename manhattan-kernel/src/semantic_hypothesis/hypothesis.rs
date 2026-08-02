use crate::predicate::Predicate;
use crate::sandbox::operators::Transformation;

pub struct SemanticStep {
    pub condition: Option<Vec<Box<dyn Predicate>>>,
    pub transformation: Transformation,
    pub target_spec: Option<crate::abstraction::program::TargetSpec>,
}

impl std::fmt::Debug for SemanticStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticStep")
            .field("condition", &self.condition.as_ref().map(|cs| cs.iter().map(|c| c.name()).collect::<Vec<_>>()))
            .field("transformation", &self.transformation)
            .field("target_spec", &self.target_spec)
            .finish()
    }
}

impl Clone for SemanticStep {
    fn clone(&self) -> Self {
        SemanticStep {
            condition: self.condition.as_ref().map(|cs| cs.iter().map(|c| c.clone_box()).collect()),
            transformation: self.transformation.clone(),
            target_spec: self.target_spec.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemanticHypothesis {
    pub steps: Vec<SemanticStep>,
    pub score: f64,
    pub num_consistent_pairs: usize,
}

impl SemanticHypothesis {
    pub fn new(steps: Vec<SemanticStep>, num_pairs: usize) -> Self {
        Self { steps, score: 0.0, num_consistent_pairs: num_pairs }
    }
}
