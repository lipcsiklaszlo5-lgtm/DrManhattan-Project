use crate::sandbox::operators::Transformation;
use crate::predicate::{Predicate, PredicateResult};
use crate::predicate::builtin::ColorPredicate;
use crate::structure::KernelStructureGraph;

#[derive(Clone)]
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
    Predicate(Box<dyn Predicate>),
}

impl std::fmt::Debug for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::AlwaysTrue => write!(f, "AlwaysTrue"),
            Condition::NodeHasAttribute(a, v) => write!(f, "NodeHasAttribute({}, {})", a, v),
            Condition::ColorEquals(c) => write!(f, "ColorEquals({})", c),
            Condition::PositionAbove(id) => write!(f, "PositionAbove({})", id),
            Condition::PositionLeftOf(id) => write!(f, "PositionLeftOf({})", id),
            Condition::Unique(s) => write!(f, "Unique({})", s),
            Condition::ExtremeByAttribute { attribute, mode } => write!(f, "ExtremeByAttribute({}, {:?})", attribute, mode),
            Condition::TouchesBorder => write!(f, "TouchesBorder"),
            Condition::And(conds) => write!(f, "And({:?})", conds),
            Condition::Not(cond) => write!(f, "Not({:?})", cond),
            Condition::StructuralRole(r) => write!(f, "StructuralRole({})", r),
            Condition::Predicate(p) => write!(f, "Predicate({})", p.name()),
        }
    }
}

impl PartialEq for Condition {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Condition::AlwaysTrue, Condition::AlwaysTrue) => true,
            (Condition::NodeHasAttribute(a1, v1), Condition::NodeHasAttribute(a2, v2)) => a1 == a2 && v1 == v2,
            (Condition::ColorEquals(c1), Condition::ColorEquals(c2)) => c1 == c2,
            (Condition::PositionAbove(id1), Condition::PositionAbove(id2)) => id1 == id2,
            (Condition::PositionLeftOf(id1), Condition::PositionLeftOf(id2)) => id1 == id2,
            (Condition::Unique(s1), Condition::Unique(s2)) => s1 == s2,
            (Condition::ExtremeByAttribute { attribute: a1, mode: m1 }, Condition::ExtremeByAttribute { attribute: a2, mode: m2 }) => a1 == a2 && m1 == m2,
            (Condition::TouchesBorder, Condition::TouchesBorder) => true,
            (Condition::And(v1), Condition::And(v2)) => v1 == v2,
            (Condition::Not(b1), Condition::Not(b2)) => b1 == b2,
            (Condition::StructuralRole(r1), Condition::StructuralRole(r2)) => r1 == r2,
            (Condition::Predicate(p1), Condition::Predicate(p2)) => p1.name() == p2.name(),
            _ => false,
        }
    }
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

// --- Predicate implementáció a Condition-höz ---
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
            Condition::Predicate(p) => p.evaluate(graph),
            _ => PredicateResult::Bool(false),
        }
    }

    fn name(&self) -> &str { "Condition" }

    fn clone_box(&self) -> Box<dyn Predicate> {
        Box::new(self.clone())
    }
}
