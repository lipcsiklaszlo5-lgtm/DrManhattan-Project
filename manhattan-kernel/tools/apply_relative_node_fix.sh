#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. transform.rs: Condition enum kibővítése Predicate variánssal, kézi trait-ek
cat > src/abstraction/transform.rs << 'EOF'
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
EOF

# 2. program.rs: RelativeToPredicate eltávolítása, PartialEq visszaállítása
python3 << 'PYEOF'
prog_path = "/workspaces/DrManhattan-Project/manhattan-kernel/src/abstraction/program.rs"
with open(prog_path, 'r') as f:
    prog = f.read()

# Eltávolítjuk a RelativeToPredicate sort
prog = prog.replace("    /// Új: predikátum-alapú relatív pozícionálás (SHE-kompatibilis)\n    RelativeToPredicate { predicate: Box<dyn Predicate>, dx_offset: i64, dy_offset: i64 },\n", "")

# Visszaállítjuk a PartialEq-et (most már a Condition implementálja)
prog = prog.replace("#[derive(Debug, Clone)]\npub enum TargetSpec {", "#[derive(Debug, Clone, PartialEq)]\npub enum TargetSpec {")

# A resolve_target_spec-ből is eltávolítjuk a RelativeToPredicate ágat
old_resolve = """            TargetSpec::RelativeToPredicate { predicate, dx_offset, dy_offset } => {
                let refs = Self::matching_nodes(graph, predicate.as_ref());
                if let Some(ref_node) = refs.first() {
                    let rx: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let ry: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                    Some((rx + dx_offset, ry + dy_offset, None))
                } else { None }
            }"""
prog = prog.replace(old_resolve, "")

with open(prog_path, 'w') as f:
    f.write(prog)
print("program.rs cleaned")
PYEOF

# 3. generator.rs: RelativeToPredicate helyett Condition::Predicate + TargetSpec::RelativeToNode
python3 << 'PYEOF'
gen_path = "/workspaces/DrManhattan-Project/manhattan-kernel/src/semantic_hypothesis/generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

# Kicseréljük a RelativeToPredicate-es blokkot
old_block = """                        match grid_anchor_for_node(node_out, grid_width, grid_height) {
                            Some(spec) => (Transformation::SemanticTranslateToTarget, Some(spec)),
                            None => {
                                // Próbáljunk relatív célpontot generálni egy referencia objektumhoz képest
                                if let Some(ref_node) = input.nodes.iter()
                                    .filter(|n| n.id != *node_id)
                                    .max_by_key(|n| n.attributes.get("area").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0))
                                {
                                    if let Some(ref_preds) = describe_node_all(&ref_node.id, input).into_iter().next() {
                                        let ref_x: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                                        let ref_y: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                                        let ax: i64 = node_out.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                                        let ay: i64 = node_out.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                                        let rel_dx = ax - ref_x;
                                        let rel_dy = ay - ref_y;
                                        let ref_pred: Box<dyn Predicate> = if ref_preds.len() == 1 {
                                            ref_preds[0].clone_box()
                                        } else {
                                            Box::new(crate::predicate::builtin::AndPredicate {
                                                predicates: ref_preds.iter().map(|p| p.clone_box()).collect(),
                                            })
                                        };
                                        (Transformation::SemanticTranslateToTarget, Some(TargetSpec::RelativeToPredicate {
                                            predicate: ref_pred,
                                            dx_offset: rel_dx,
                                            dy_offset: rel_dy,
                                        }))
                                    } else {
                                        continue
                                    }
                                } else {
                                    continue
                                }
                            }
                        }"""

new_block = """                        match grid_anchor_for_node(node_out, grid_width, grid_height) {
                            Some(spec) => (Transformation::SemanticTranslateToTarget, Some(spec)),
                            None => {
                                // Próbáljunk relatív célpontot generálni egy referencia objektumhoz képest
                                if let Some(ref_node) = input.nodes.iter()
                                    .filter(|n| n.id != *node_id)
                                    .max_by_key(|n| n.attributes.get("area").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0))
                                {
                                    if let Some(ref_preds) = describe_node_all(&ref_node.id, input).into_iter().next() {
                                        let ref_x: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                                        let ref_y: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                                        let ax: i64 = node_out.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                                        let ay: i64 = node_out.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                                        let rel_dx = ax - ref_x;
                                        let rel_dy = ay - ref_y;
                                        let ref_pred: Box<dyn Predicate> = if ref_preds.len() == 1 {
                                            ref_preds[0].clone_box()
                                        } else {
                                            Box::new(crate::predicate::builtin::AndPredicate {
                                                predicates: ref_preds.iter().map(|p| p.clone_box()).collect(),
                                            })
                                        };
                                        let condition = Condition::Predicate(ref_pred);
                                        (Transformation::SemanticTranslateToTarget, Some(TargetSpec::RelativeToNode {
                                            condition: Box::new(condition),
                                            dx_offset: rel_dx,
                                            dy_offset: rel_dy,
                                        }))
                                    } else {
                                        continue
                                    }
                                } else {
                                    continue
                                }
                            }
                        }"""

gen = gen.replace(old_block, new_block)

# Frissítjük a step_signature függvényt is: RelativeToPredicate helyett RelativeToNode
gen = gen.replace(
    "        Some(TargetSpec::RelativeToPredicate { predicate, .. }) => {\n            format!(\"RelativeToPredicate:{}\", predicate.name())\n        }",
    "        Some(TargetSpec::RelativeToNode { condition, dx_offset, dy_offset }) => {\n            format!(\"RelativeToNode:{}_{}_{}\", condition.name(), dx_offset, dy_offset)\n        }"
)

with open(gen_path, 'w') as f:
    f.write(gen)
print("generator.rs updated")
PYEOF

# 4. Build & test
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -10
echo "===== COVERAGE 017c7c7b ====="
target/release/arc_abstraction_coverage ARC-AGI-master/data/training/017c7c7b.json 2>&1
echo "===== COMMIT ====="
git add -A && git commit -m "feat: use Condition::Predicate in TargetSpec::RelativeToNode for semantic translate" && git push
