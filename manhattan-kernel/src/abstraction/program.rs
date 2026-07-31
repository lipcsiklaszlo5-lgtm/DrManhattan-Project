use crate::sandbox::operators::Transformation;
use crate::structure::KernelStructureGraph;
use crate::structure::topology::{graph_diff, NodeTransformation};
use super::transform::{TransformRule, Condition, TransformationAlgebra};

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
}

impl ProgramSynthesizer {
    pub fn new() -> Self {
        Self { programs: Vec::new(), algebra: TransformationAlgebra::new() }
    }

    pub fn learn_from_example(&mut self, before: &KernelStructureGraph, after: &KernelStructureGraph) -> Option<Program> {
        let diffs = graph_diff(before, after);
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

    pub fn find_best_program(&self, graph: &KernelStructureGraph, target: &KernelStructureGraph) -> Option<&Program> {
        self.programs.iter()
            .filter(|p| {
                let result = p.apply(graph);
                result.nodes.len() == target.nodes.len()
            })
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
    }
}
