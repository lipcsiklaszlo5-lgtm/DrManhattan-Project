use crate::structure::KernelStructureGraph;
use crate::structure::topology::graph_diff;
use crate::concept::{Concept, ConceptDetector, ConceptRegistry};
use std::collections::HashMap;

/// Dinamikusan generált detektor, ami egy adott színű csomópontot keres.
struct ColorDetector {
    color: String,
    concept: Concept,
}

impl ConceptDetector for ColorDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        if graph.nodes.iter().any(|n| n.attributes.get("color").map_or(false, |c| c == &self.color)) {
            vec![self.concept.clone()]
        } else {
            vec![]
        }
    }
}

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
        registry: &mut ConceptRegistry,
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
                                    self.discovered.push(concept.clone());
                                    if let Some(detector) = Self::create_detector(&pattern, concept.clone()) {
                                        registry.add_detector(Box::new(detector));
                                    }
                                }
                                // Mindenképpen adjuk hozzá a learned_concepts-hez, hogy a scan mindig visszaadja
                                registry.add_concept(concept);
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

    fn create_detector(pattern: &str, concept: Concept) -> Option<ColorDetector> {
        let parts: Vec<&str> = pattern.split('_').collect();
        if parts.len() >= 2 {
            let color = parts[1..].join("_");
            Some(ColorDetector { color, concept })
        } else {
            None
        }
    }
}
