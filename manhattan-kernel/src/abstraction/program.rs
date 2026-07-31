use crate::sandbox::operators::Transformation;
use crate::structure::KernelStructureGraph;
use crate::structure::topology::{graph_diff, NodeTransformation};
use super::transform::{TransformRule, Condition, TransformationAlgebra};
use crate::concept::Concept;

#[derive(Debug, Clone)]
pub struct Program {
    pub steps: Vec<Transformation>,
    pub conditions: Vec<Condition>,
    pub confidence: f32,
    pub success_count: u64,
}

impl Program {
    pub fn new(steps: Vec<Transformation>) -> Self {
        Self { steps, conditions: vec![Condition::AlwaysTrue], confidence: 0.5, success_count: 1 }
    }

    pub fn cost(&self) -> f64 {
        (1.0 - self.confidence as f64) * (self.steps.len() as f64).max(1.0)
    }

    pub fn apply(&self, graph: &KernelStructureGraph) -> KernelStructureGraph {
        let mut current = graph.clone();
        for step in &self.steps {
            current = crate::sandbox::operators::apply_transformation(&current, step);
        }
        current
    }

    pub fn record_success(&mut self) {
        self.success_count += 1;
        self.confidence = (self.confidence + 0.1).min(1.0);
    }

    pub fn record_failure(&mut self) {
        self.confidence = (self.confidence - 0.1).max(0.0);
    }
}

pub struct ProgramSynthesizer {
    pub programs: Vec<Program>,
    pub algebra: TransformationAlgebra,
    concept_transforms: Vec<(Concept, String)>,
}

impl ProgramSynthesizer {
    pub fn new() -> Self {
        let mut ct = Vec::new();
        // Alapértelmezett fogalom-transzformáció hozzárendelések (core knowledge)
        ct.push((Concept::Connected, "Translate".into()));
        ct.push((Concept::Symmetry, "Recolor".into()));
        ct.push((Concept::Player, "Translate".into()));
        ct.push((Concept::Exit, "Translate".into()));
        ct.push((Concept::Hole, "Delete".into()));
        ct.push((Concept::Largest, "Translate".into()));
        Self { programs: Vec::new(), algebra: TransformationAlgebra::new(), concept_transforms: ct }
    }

    pub fn map_concept_to_transform(&mut self, concept: Concept, transform_name: &str) {
        self.concept_transforms.push((concept, transform_name.to_string()));
    }

    pub fn transforms_for_concept(&self, concept: &Concept) -> Vec<String> {
        self.concept_transforms.iter()
            .filter(|(c, _)| c == concept)
            .map(|(_, t)| t.clone())
            .collect()
    }

    pub fn learn_from_example(&mut self, before: &KernelStructureGraph, after: &KernelStructureGraph) -> Option<Program> {
        let diffs: Vec<_> = graph_diff(before, after).into_iter().filter(|d| !matches!(d, NodeTransformation::Unchanged { .. })).collect();
        if diffs.is_empty() { return None; }

        let mut steps = Vec::new();
        for diff in &diffs {
            match diff {
                NodeTransformation::Translate { node_id, dx, dy } => {
                    steps.push(Transformation::Translate { node_id: node_id.clone(), dx: *dx, dy: *dy });
                }
                NodeTransformation::Recolor { node_id, new_color } => {
                    steps.push(Transformation::Recolor { node_id: node_id.clone(), new_color: new_color.clone() });
                }
                NodeTransformation::Delete { node_id } => {
                    steps.push(Transformation::Delete { node_id: node_id.clone() });
                }
                NodeTransformation::Create { node_id: nid, color, bbox_x, bbox_y } => {
                    if let Some(new_node) = after.nodes.iter().find(|n| n.id == *nid) {
                        let bw = new_node.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1);
                        let bh = new_node.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1);
                        steps.push(Transformation::Create { color: color.clone(), bbox_x: *bbox_x, bbox_y: *bbox_y, bbox_w: bw, bbox_h: bh });
                    }
                }
                NodeTransformation::Rotate { node_id, angle } => {
                    steps.push(Transformation::Rotate { node_id: node_id.clone(), angle: *angle });
                }
                NodeTransformation::Unchanged { .. } => {}
            }
        }

        if steps.is_empty() { return None; }

        let program = Program::new(steps);
        self.programs.push(program.clone());
        for step in &program.steps {
            self.algebra.add_rule(TransformRule::new(step.clone()));
        }

        Some(program)
    }

    pub fn generalize_from_pairs(&mut self, pairs: &[(KernelStructureGraph, KernelStructureGraph)]) -> Option<Program> {
        if pairs.is_empty() { return None; }

        let mut all_operator_types: Vec<Vec<String>> = Vec::new();

        for (before, after) in pairs {
            let diffs: Vec<_> = graph_diff(before, after).into_iter().filter(|d| !matches!(d, NodeTransformation::Unchanged { .. })).collect();
            let op_types: Vec<String> = diffs.iter().map(|d| match d {
                NodeTransformation::Translate { .. } => "Translate".to_string(),
                NodeTransformation::Recolor { .. } => "Recolor".to_string(),
                NodeTransformation::Delete { .. } => "Delete".to_string(),
                NodeTransformation::Create { .. } => "Create".to_string(),
                NodeTransformation::Rotate { .. } => "Rotate".to_string(),
                NodeTransformation::Unchanged { .. } => "Unchanged".to_string(),
            }).collect();
            all_operator_types.push(op_types);
        }

        let common_types = Self::intersect_sequences(&all_operator_types);

        if common_types.is_empty() {
            let (before, after) = &pairs[0];
            return self.learn_from_example(before, after);
        }

        let (before, after) = &pairs[0];
        let diffs: Vec<_> = graph_diff(before, after).into_iter().filter(|d| !matches!(d, NodeTransformation::Unchanged { .. })).collect();

        let mut steps = Vec::new();
        for (i, diff) in diffs.iter().enumerate() {
            if i < common_types.len() {
                match diff {
                    NodeTransformation::Recolor { .. } => {
                        steps.push(Transformation::RecolorToTarget { node_id: "obj_0".to_string() });
                    }
                    NodeTransformation::Translate { .. } => {
                        steps.push(Transformation::TranslateToTarget { node_id: "obj_0".to_string() });
                    }
                    NodeTransformation::Rotate { angle, .. } => {
                        steps.push(Transformation::Rotate { node_id: "obj_0".to_string(), angle: *angle });
                    }
                    _ => {
                        if let Some(step) = Self::diff_to_step(diff, after) {
                            steps.push(step);
                        }
                    }
                }
            }
        }

        if steps.is_empty() { return None; }

        let program = Program::new(steps);
        self.programs.push(program.clone());
        Some(program)
    }

    fn intersect_sequences(sequences: &[Vec<String>]) -> Vec<String> {
        if sequences.is_empty() { return vec![]; }
        let first = &sequences[0];
        first.iter().enumerate().filter(|(i, op_type)| {
            sequences.iter().all(|seq| seq.get(*i) == Some(op_type))
        }).map(|(_, op_type)| op_type.clone()).collect()
    }

    fn diff_to_step(diff: &NodeTransformation, after: &KernelStructureGraph) -> Option<Transformation> {
        match diff {
            NodeTransformation::Translate { node_id, dx, dy } => {
                Some(Transformation::Translate { node_id: node_id.clone(), dx: *dx, dy: *dy })
            }
            NodeTransformation::Recolor { node_id, new_color } => {
                Some(Transformation::Recolor { node_id: node_id.clone(), new_color: new_color.clone() })
            }
            NodeTransformation::Delete { node_id } => {
                Some(Transformation::Delete { node_id: node_id.clone() })
            }
            NodeTransformation::Create { node_id: nid, color, bbox_x, bbox_y } => {
                if let Some(new_node) = after.nodes.iter().find(|n| n.id == *nid) {
                    let bw = new_node.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1);
                    let bh = new_node.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1);
                    Some(Transformation::Create { color: color.clone(), bbox_x: *bbox_x, bbox_y: *bbox_y, bbox_w: bw, bbox_h: bh })
                } else {
                    None
                }
            }
            NodeTransformation::Rotate { node_id, angle } => {
                Some(Transformation::Rotate { node_id: node_id.clone(), angle: *angle })
            }
            NodeTransformation::Unchanged { .. } => None,
        }
    }

    pub fn find_best_program(&self, graph: &KernelStructureGraph, target: &KernelStructureGraph) -> Option<&Program> {
        self.programs.iter()
            .filter(|p| {
                let result = p.apply(graph);
                result.nodes.len() == target.nodes.len()
            })
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
    }
}
