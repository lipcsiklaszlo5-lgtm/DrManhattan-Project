#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. Teljes generate_candidate_steps átírása a többreferenciás logikával
python3 << 'PYEOF'
gen_path = "src/semantic_hypothesis/generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

# Kicseréljük a generate_candidate_steps függvényt
old_func_start = "pub fn generate_candidate_steps("
old_func_end = "    steps\n}"
new_func = """pub fn generate_candidate_steps(
    input: &KernelStructureGraph,
    output: &KernelStructureGraph,
    grid_width: u8,
    grid_height: u8,
) -> Vec<SemanticStep> {
    let diffs = graph_diff(input, output);
    let mut steps = Vec::new();

    for diff in diffs {
        match &diff {
            NodeTransformation::Translate { node_id, .. } => {
                let node_out = match output.nodes.iter().find(|n| n.id == *node_id) {
                    Some(n) => n,
                    None => continue,
                };
                let tr = abstract_translate(node_id, input, output, grid_width, grid_height);

                // 1. Próbáljunk GridAnchor-t
                if let Some(spec) = grid_anchor_for_node(node_out, grid_width, grid_height) {
                    let all_descriptions = describe_node_all(node_id, input);
                    for preds in all_descriptions {
                        steps.push(SemanticStep {
                            condition: Some(preds),
                            transformation: tr.clone(),
                            target_spec: Some(spec.clone()),
                        });
                    }
                    continue;
                }

                // 2. Több referenciaobjektum-hipotézis
                let ref_predicates: Vec<Box<dyn Predicate>> = vec![
                    Box::new(crate::predicate::builtin::LargestPredicate),
                    Box::new(crate::predicate::builtin::SmallestPredicate),
                    Box::new(crate::predicate::builtin::LeftmostPredicate),
                    Box::new(crate::predicate::builtin::RightmostPredicate),
                    Box::new(crate::predicate::builtin::TopmostPredicate),
                    Box::new(crate::predicate::builtin::BottommostPredicate),
                    Box::new(crate::predicate::builtin::MajorityColorPredicate),
                    Box::new(crate::predicate::builtin::MinorityColorPredicate),
                    Box::new(crate::predicate::builtin::UniqueColorPredicate),
                ];
                for ref_predicate in ref_predicates {
                    let result = ObjectSelector::select(
                        ref_predicate.as_ref(),
                        input,
                        &crate::object_selector::SelectionStrategy::Best,
                        None,
                    );
                    if let Some(ref_node) = result.selected.first() {
                        if let Some(ref_node) = input.nodes.iter().find(|n| n.id == ref_node.node_id) {
                            if ref_node.id == *node_id {
                                continue; // a referenciaobjektum nem lehet maga a mozgó objektum
                            }
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
                                let target_spec = TargetSpec::RelativeToNode {
                                    condition: Box::new(condition),
                                    dx_offset: rel_dx,
                                    dy_offset: rel_dy,
                                };
                                let all_descriptions = describe_node_all(node_id, input);
                                for preds in all_descriptions {
                                    steps.push(SemanticStep {
                                        condition: Some(preds),
                                        transformation: tr.clone(),
                                        target_spec: Some(target_spec.clone()),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            NodeTransformation::Recolor { node_id, new_color } => {
                let spec = TargetSpec::Constant(new_color.clone());
                let all_descriptions = describe_node_all(node_id, input);
                for preds in all_descriptions {
                    steps.push(SemanticStep {
                        condition: Some(preds),
                        transformation: Transformation::SemanticRecolorToTarget,
                        target_spec: Some(spec.clone()),
                    });
                }
            }
            NodeTransformation::Delete { node_id } => {
                let all_descriptions = describe_node_all(node_id, input);
                for preds in all_descriptions {
                    steps.push(SemanticStep {
                        condition: Some(preds),
                        transformation: Transformation::Delete { node_id: String::new() },
                        target_spec: None,
                    });
                }
            }
            _ => continue,
        }
    }
    steps
}"""

# Kicseréljük a régi függvényt
import re
pattern = re.compile(r'pub fn generate_candidate_steps\(.*?steps\n\}', re.DOTALL)
gen = pattern.sub(new_func, gen)

with open(gen_path, 'w') as f:
    f.write(gen)
print("generate_candidate_steps replaced with multi-reference version")
PYEOF

# 2. Build & teszt
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -15
echo "===== COVERAGE 05f2a901 ====="
MK_DIAG=1 target/release/arc_abstraction_coverage ARC-AGI-master/data/training/05f2a901.json 2>&1 | grep -E "ACCEPTED|returning|coverage"
echo "===== COMMIT ====="
git add -A && git commit -m "feat: multi-reference hypotheses in generate_candidate_steps" && git push
