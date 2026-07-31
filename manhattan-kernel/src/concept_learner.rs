use crate::structure::KernelStructureGraph;
use crate::structure::topology::graph_diff;
use crate::concept::{Concept, ConceptRegistry};
use std::collections::HashMap;

pub struct ConceptLearner {
    pub discovered: Vec<Concept>,
    pattern_counts: HashMap<String, usize>,
}

impl ConceptLearner {
    pub fn new() -> Self {
        Self {
            discovered: Vec::new(),
            pattern_counts: HashMap::new(),
        }
    }

    pub fn learn_from_diff(
        &mut self,
        before: &KernelStructureGraph,
        after: &KernelStructureGraph,
        registry: &ConceptRegistry,
    ) -> Vec<Concept> {
        let diffs = graph_diff(before, after);
        let mut new_concepts = Vec::new();

        for diff in &diffs {
            match diff {
                crate::structure::topology::NodeTransformation::Unchanged { .. } => {}
                _ => {
                    if let Some(pattern) = Self::extract_pattern(diff) {
                        let count = self.pattern_counts.entry(pattern.clone()).or_insert(0);
                        *count += 1;
                        if *count >= 2 {
                            if let Some(concept) = Self::pattern_to_concept(&pattern) {
                                if !registry.scan(after).contains(&concept) {
                                    new_concepts.push(concept.clone());
                                    self.discovered.push(concept);
                                }
                            }
                        }
                    }
                }
            }
        }
        new_concepts
    }

    fn extract_pattern(
        diff: &crate::structure::topology::NodeTransformation,
    ) -> Option<String> {
        match diff {
            crate::structure::topology::NodeTransformation::Create { color, .. } => {
                Some(format!("create_{}", color))
            }
            crate::structure::topology::NodeTransformation::Recolor { new_color, .. } => {
                Some(format!("recolor_{}", new_color))
            }
            crate::structure::topology::NodeTransformation::Translate { .. } => {
                Some("translate".to_string())
            }
            crate::structure::topology::NodeTransformation::Rotate { .. } => {
                Some("rotate".to_string())
            }
            crate::structure::topology::NodeTransformation::Delete { .. } => {
                Some("delete".to_string())
            }
            _ => None,
        }
    }

    fn pattern_to_concept(pattern: &str) -> Option<Concept> {
        if pattern.contains("create") {
            Some(Concept::Connected)
        } else if pattern.contains("recolor") {
            Some(Concept::Symmetry)
        } else if pattern.contains("translate") {
            Some(Concept::Player)
        } else {
            None
        }
    }
}
