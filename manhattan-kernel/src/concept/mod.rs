pub mod detectors;

use crate::structure::KernelStructureGraph;
use crate::predicate::Predicate;
use crate::predicate::builtin::{SymmetryPredicate, HolePredicate, BorderObjectPredicate, CrossPredicate, LargestPredicate};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Concept {
    Hole, Border, Cross, Symmetry, Connected, Largest, Smallest,
    Player, Exit, Key, Door, Button, Obstacle,
}

pub trait ConceptDetector { fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept>; }

pub struct ConceptRegistry {
    detectors: Vec<Box<dyn ConceptDetector>>,
    learned_concepts: Vec<Concept>,
}

impl Clone for ConceptRegistry {
    fn clone(&self) -> Self {
        Self { detectors: Vec::new(), learned_concepts: self.learned_concepts.clone() }
    }
}

impl ConceptRegistry {
    pub fn new() -> Self { Self { detectors: Vec::new(), learned_concepts: Vec::new() } }
    pub fn add_detector(&mut self, det: Box<dyn ConceptDetector>) { self.detectors.push(det); }
    pub fn add_concept(&mut self, concept: Concept) { self.learned_concepts.push(concept); }
    pub fn scan(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        let mut results = Vec::new();
        for det in &self.detectors { results.extend(det.detect(graph)); }
        results.extend(self.learned_concepts.clone());
        results.sort(); results.dedup(); results
    }
    pub fn consolidate(&mut self) { self.learned_concepts.sort(); self.learned_concepts.dedup(); }
    pub fn to_predicates(&self, graph: &KernelStructureGraph) -> Vec<Box<dyn Predicate>> {
        let concepts = self.scan(graph);
        let mut preds: Vec<Box<dyn Predicate>> = Vec::new();
        for c in concepts {
            match c {
                Concept::Symmetry => preds.push(Box::new(SymmetryPredicate)),
                Concept::Hole => preds.push(Box::new(HolePredicate)),
                Concept::Border => preds.push(Box::new(BorderObjectPredicate)),
                Concept::Cross => preds.push(Box::new(CrossPredicate)),
                Concept::Largest => preds.push(Box::new(LargestPredicate)),
                _ => {}
            }
        }
        preds
    }
}

impl Default for ConceptRegistry {
    fn default() -> Self {
        let mut reg = Self::new();
        reg.add_detector(Box::new(detectors::BorderDetector));
        reg.add_detector(Box::new(detectors::HoleDetector));
        reg.add_detector(Box::new(detectors::SymmetryDetector));
        reg.add_detector(Box::new(detectors::LargestObjectDetector));
        reg.add_detector(Box::new(detectors::CrossDetector));
        reg.add_detector(Box::new(detectors::RoleDetector));
        reg.add_detector(Box::new(detectors::MirrorDetector));
        reg.add_detector(Box::new(detectors::ContainmentDetector));
        reg.add_detector(Box::new(detectors::AdjacencyDetector));
        reg.add_detector(Box::new(detectors::PatternDetector));
        reg.add_detector(Box::new(detectors::CauseEffectDetector));
        reg.add_detector(Box::new(detectors::SequenceDetector));
        reg.add_detector(Box::new(detectors::ObjectCountDetector));
        reg.add_detector(Box::new(detectors::FillDetector));
        reg
    }
}
