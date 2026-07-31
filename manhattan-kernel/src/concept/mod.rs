pub mod detectors;

use crate::structure::KernelStructureGraph;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Concept {
    Hole,
    Border,
    Cross,
    Symmetry,
    Connected,
    Largest,
    Smallest,
    Player,
    Exit,
    Key,
    Door,
    Button,
    Obstacle,
}

pub trait ConceptDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept>;
}

pub struct ConceptRegistry {
    detectors: Vec<Box<dyn ConceptDetector>>,
}

impl ConceptRegistry {
    pub fn new() -> Self { Self { detectors: Vec::new() } }
    pub fn add_detector(&mut self, det: Box<dyn ConceptDetector>) { self.detectors.push(det); }
    pub fn scan(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        let mut results = Vec::new();
        for det in &self.detectors { results.extend(det.detect(graph)); }
        results.sort(); results.dedup(); results
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
        reg
    }
}
