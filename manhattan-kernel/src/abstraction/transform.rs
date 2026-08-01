use crate::sandbox::operators::Transformation;
use crate::predicate::{Predicate, PredicateResult};
use crate::predicate::builtin::ColorPredicate;
use crate::structure::KernelStructureGraph;

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    AlwaysTrue,
    NodeHasAttribute(String, String),
    ColorEquals(String),
    PositionAbove(String),
    PositionLeftOf(String),
    Unique(String),
    ExtremeByAttribute { attribute: String, mode: ExtremeMode },
    TouchesBorder,
    And(Vec<Condition>),
    Not(Box<Condition>),
    StructuralRole(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExtremeMode { Max, Min }

#[derive(Debug, Clone)]
pub struct TransformRule {
    pub transformation: Transformation,
}

impl TransformRule {
    pub fn new(transformation: Transformation) -> Self { Self { transformation } }
}

pub struct TransformationAlgebra {
    pub rules: Vec<TransformRule>,
}

impl TransformationAlgebra {
    pub fn new() -> Self { Self { rules: Vec::new() } }
    pub fn add_rule(&mut self, rule: TransformRule) { self.rules.push(rule); }
}

// --- Predicate implementáció a Condition-höz (most már clone_box-szal) ---
impl Predicate for Condition {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        match self {
            Condition::AlwaysTrue => PredicateResult::Bool(true),
            Condition::NodeHasAttribute(attr, val) => {
                let matching: Vec<(String, f32)> = graph.nodes.iter()
                    .filter(|n| n.attributes.get(attr) == Some(val))
                    .map(|n| (n.id.clone(), 1.0))
                    .collect();
                PredicateResult::RankedList(matching)
            }
            Condition::ColorEquals(color) => {
                ColorPredicate { color: color.clone() }.evaluate(graph)
            }
            Condition::PositionAbove(other_id) => {
                let ref_node = graph.nodes.iter().find(|n| &n.id == other_id);
                if let Some(rn) = ref_node {
                    let ref_y: i64 = rn.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let matching: Vec<(String, f32)> = graph.nodes.iter()
                        .filter(|n| {
                            let y: i64 = n.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                            let h: i64 = n.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(0);
                            y + h <= ref_y
                        })
                        .map(|n| (n.id.clone(), 1.0))
                        .collect();
                    PredicateResult::RankedList(matching)
                } else {
                    PredicateResult::Bool(false)
                }
            }
            Condition::PositionLeftOf(other_id) => {
                let ref_node = graph.nodes.iter().find(|n| &n.id == other_id);
                if let Some(rn) = ref_node {
                    let ref_x: i64 = rn.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let matching: Vec<(String, f32)> = graph.nodes.iter()
                        .filter(|n| {
                            let x: i64 = n.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                            let w: i64 = n.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(0);
                            x + w <= ref_x
                        })
                        .map(|n| (n.id.clone(), 1.0))
                        .collect();
                    PredicateResult::RankedList(matching)
                } else {
                    PredicateResult::Bool(false)
                }
            }
            _ => PredicateResult::Bool(false),
        }
    }

    fn name(&self) -> &str { "Condition" }

    fn clone_box(&self) -> Box<dyn Predicate> {
        Box::new(self.clone())
    }
}
