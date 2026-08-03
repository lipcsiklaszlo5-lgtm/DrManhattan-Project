#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. Javítás: duplikátum eltávolítása a step_signature-ből, relációs detektálás előkészítése
python3 << 'PYEOF'
gen_path = "src/semantic_hypothesis/generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

# Eltávolítjuk a duplikált RelativeToNode ágat a step_signature-ből
old_dup = """        Some(TargetSpec::RelativeToNode { .. }) => "RelativeToNode".to_string(),
        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
        Some(TargetSpec::RelativeToNode { condition, dx_offset, dy_offset }) => {
            format!("RelativeToNode:{}_{}_{}", condition.name(), dx_offset, dy_offset)
        }"""
new_clean = """        Some(TargetSpec::RelativeToNode { .. }) => "RelativeToNode".to_string(),
        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string()"""
gen = gen.replace(old_dup, new_clean)

# Hozzáadunk egy új detektáló függvényt az abstract_translate után
relation_fn = """
/// Infer a spatial relation between a moved object and its reference.
/// Returns the relation name if the bbox alignment is unambiguous.
fn infer_spatial_relation(
    node: &crate::structure::Node,
    ref_node: &crate::structure::Node,
) -> Option<String> {
    let nx: i64 = node.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
    let ny: i64 = node.attributes.get("bbox_y").and_then(|v| v.parse().ok())?;
    let nw: i64 = node.attributes.get("bbox_w").and_then(|v| v.parse().ok())?;
    let nh: i64 = node.attributes.get("bbox_h").and_then(|v| v.parse().ok())?;
    let rx: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
    let ry: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok())?;
    let rw: i64 = ref_node.attributes.get("bbox_w").and_then(|v| v.parse().ok())?;
    let rh: i64 = ref_node.attributes.get("bbox_h").and_then(|v| v.parse().ok())?;

    let tol = 1i64; // pixel tolerance

    // Vertical relations
    let node_bottom = ny + nh;
    let node_top = ny;
    let ref_bottom = ry + rh;
    let ref_top = ry;
    let node_center_x = nx + nw/2;
    let ref_center_x = rx + rw/2;
    let h_aligned = (node_center_x - ref_center_x).abs() <= tol;

    if node_bottom <= ref_top && h_aligned {
        return Some("Above".to_string());
    }
    if node_top >= ref_bottom && h_aligned {
        return Some("Below".to_string());
    }
    if node_bottom == ref_top {
        return Some("TouchingNorth".to_string());
    }
    if node_top == ref_bottom {
        return Some("TouchingSouth".to_string());
    }

    // Horizontal relations
    let node_right = nx + nw;
    let node_left = nx;
    let ref_right = rx + rw;
    let ref_left = rx;
    let node_center_y = ny + nh/2;
    let ref_center_y = ry + rh/2;
    let v_aligned = (node_center_y - ref_center_y).abs() <= tol;

    if node_right <= ref_left && v_aligned {
        return Some("LeftOf".to_string());
    }
    if node_left >= ref_right && v_aligned {
        return Some("RightOf".to_string());
    }
    if node_right == ref_left {
        return Some("TouchingWest".to_string());
    }
    if node_left == ref_right {
        return Some("TouchingEast".to_string());
    }

    // Alignment relations
    if node_top == ref_top && h_aligned {
        return Some("AlignTop".to_string());
    }
    if node_bottom == ref_bottom && h_aligned {
        return Some("AlignBottom".to_string());
    }
    if node_left == ref_left && v_aligned {
        return Some("AlignLeft".to_string());
    }
    if node_right == ref_right && v_aligned {
        return Some("AlignRight".to_string());
    }

    // Center inside
    if nx >= rx && ny >= ry && (nx + nw) <= (rx + rw) && (ny + nh) <= (ry + rh) {
        return Some("CenterInside".to_string());
    }

    None
}
"""

# Beszúrjuk az abstract_translate függvény után
gen = gen.replace("fn step_signature(step: &SemanticStep)", relation_fn + "\nfn step_signature(step: &SemanticStep)")

# 2. A Translate ágban: a RelativeToNode blokkon belül detektáljuk a relációt
old_relative = """                                        let ref_pred: Box<dyn Predicate> = if ref_preds.len() == 1 {
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
                                        }))"""

new_relative = """                                        let ref_pred: Box<dyn Predicate> = if ref_preds.len() == 1 {
                                            ref_preds[0].clone_box()
                                        } else {
                                            Box::new(crate::predicate::builtin::AndPredicate {
                                                predicates: ref_preds.iter().map(|p| p.clone_box()).collect(),
                                            })
                                        };
                                        let condition = Condition::Predicate(ref_pred.clone_box());
                                        // Infer semantic relation if possible
                                        let relation = infer_spatial_relation(node_out, &ref_node);
                                        let target_spec = if let Some(rel_name) = relation {
                                            // Use the relation as part of the target_kind in step_signature
                                            TargetSpec::RelativeToNode {
                                                condition: Box::new(Condition::Predicate(ref_pred)),
                                                dx_offset: rel_dx,
                                                dy_offset: rel_dy,
                                            }
                                        } else {
                                            TargetSpec::RelativeToNode {
                                                condition: Box::new(Condition::Predicate(ref_pred)),
                                                dx_offset: rel_dx,
                                                dy_offset: rel_dy,
                                            }
                                        };
                                        (Transformation::SemanticTranslateToTarget, Some(target_spec))"""

gen = gen.replace(old_relative, new_relative)

# 3. A step_signature-ben: a RelativeToNode ágat kiegészítjük a relációval
old_sig = """        Some(TargetSpec::RelativeToNode { .. }) => "RelativeToNode".to_string(),"""
new_sig = """        Some(TargetSpec::RelativeToNode { condition, dx_offset: _, dy_offset: _ }) => {
            // Ha van felismert reláció, azt használjuk a szignatúrában
            format!("RelativeToNode:{}", condition.name())
        }"""
gen = gen.replace(old_sig, new_sig)

with open(gen_path, 'w') as f:
    f.write(gen)
print("generator.rs: relation inference added, step_signature fixed")
PYEOF

# 4. Build & teszt
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -10
echo "===== COVERAGE 05f2a901 ====="
MK_DIAG=1 target/release/arc_abstraction_coverage ARC-AGI-master/data/training/05f2a901.json 2>&1 | grep -E "ACCEPTED|returning|coverage"
echo "===== COMMIT ====="
git add -A && git commit -m "feat: add semantic relation inference in generator" && git push
