#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# ===== 1. TargetSpec bővítése GravitateAnchor variánssal =====
python3 << 'PYEOF'
prog_path = "src/abstraction/program.rs"
with open(prog_path, 'r') as f:
    prog = f.read()

# Új variáns beszúrása a CopyAttributeFrom után
old_enum = """    CopyAttributeFrom { condition: Box<Condition>, attribute: String },
}"""
new_enum = """    CopyAttributeFrom { condition: Box<Condition>, attribute: String },
    /// Anchor for Gravitate transformation: the object to gravitate toward
    GravitateAnchor { anchor_predicate: Box<dyn Predicate> },
}"""
prog = prog.replace(old_enum, new_enum)

# resolve_target_spec kiterjesztése
old_resolve_end = """            TargetSpec::CopyAttributeFrom { condition, attribute } => {
                let refs = Self::matching_nodes(graph, condition.as_ref());
                if let Some(ref_node) = refs.first() {
                    let val = ref_node.attributes.get(attribute).cloned();
                    Some((0, 0, val))
                } else { None }
            }

        }"""
new_resolve_end = """            TargetSpec::CopyAttributeFrom { condition, attribute } => {
                let refs = Self::matching_nodes(graph, condition.as_ref());
                if let Some(ref_node) = refs.first() {
                    let val = ref_node.attributes.get(attribute).cloned();
                    Some((0, 0, val))
                } else { None }
            }
            TargetSpec::GravitateAnchor { anchor_predicate } => {
                let refs = Self::matching_nodes(graph, anchor_predicate.as_ref());
                if let Some(ref_node) = refs.first() {
                    let rx: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let ry: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                    Some((rx, ry, None))
                } else { None }
            }

        }"""
prog = prog.replace(old_resolve_end, new_resolve_end)

with open(prog_path, 'w') as f:
    f.write(prog)
print("TargetSpec::GravitateAnchor added")
PYEOF

# ===== 2. Transformation enum bővítése SemanticGravitate variánssal =====
python3 << 'PYEOF'
ops_path = "src/sandbox/operators.rs"
with open(ops_path, 'r') as f:
    ops = f.read()

# Beszúrjuk az új variánst a SemanticRecolorToTarget után
old_enum = """    SemanticRecolorToTarget,
    MirrorHorizontal { node_id: String },"""
new_enum = """    SemanticRecolorToTarget,
    SemanticGravitate,
    MirrorHorizontal { node_id: String },"""
ops = ops.replace(old_enum, new_enum)

# apply_transformation: hozzáadjuk az új variánst a NoOp ághoz
old_match = """        Transformation::SemanticMirrorHorizontal
        | Transformation::SemanticMirrorVertical
        | Transformation::SemanticTranslateToTarget
        | Transformation::SemanticRecolorToTarget => {
            // These are handled by GeneralizedProgram::apply_step with selected node IDs
        }"""
new_match = """        Transformation::SemanticMirrorHorizontal
        | Transformation::SemanticMirrorVertical
        | Transformation::SemanticTranslateToTarget
        | Transformation::SemanticRecolorToTarget
        | Transformation::SemanticGravitate => {
            // These are handled by GeneralizedProgram::apply_step with selected node IDs
        }"""
ops = ops.replace(old_match, new_match)

with open(ops_path, 'w') as f:
    f.write(ops)
print("Transformation::SemanticGravitate added")
PYEOF

# ===== 3. Gravitate függvény és apply_step kiegészítése =====
python3 << 'PYEOF'
prog_path = "src/abstraction/program.rs"
with open(prog_path, 'r') as f:
    prog = f.read()

# Gravitate függvény beszúrása az apply_step elé
gravitate_fn = """
/// Calculate the translation needed for `moving` to touch `anchor`.
/// If columns overlap -> vertical movement; if rows overlap -> horizontal.
fn gravitate(moving: &crate::structure::Node, anchor: &crate::structure::Node) -> (i64, i64) {
    let mx: i64 = moving.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
    let my: i64 = moving.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
    let mw: i64 = moving.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1);
    let mh: i64 = moving.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1);
    let ax: i64 = anchor.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
    let ay: i64 = anchor.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
    let aw: i64 = anchor.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1);
    let ah: i64 = anchor.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1);

    let col_overlap = mx < ax + aw && mx + mw > ax;  // oszlop-tartomány átfedés
    let row_overlap = my < ay + ah && my + mh > ay;  // sor-tartomány átfedés

    if col_overlap && !row_overlap {
        // Függőleges mozgás
        let dy = if my + mh <= ay {
            ay - (my + mh)  // mozgó a horgony alatt
        } else {
            ay + ah - my    // mozgó a horgony felett
        };
        (0, dy)
    } else if row_overlap && !col_overlap {
        // Vízszintes mozgás
        let dx = if mx + mw <= ax {
            ax - (mx + mw)  // mozgó a horgonytól balra
        } else {
            ax + aw - mx    // mozgó a horgonytól jobbra
        };
        (dx, 0)
    } else {
        // Ha mindkét tengelyen átfedés van, vagy egyiken sem, nincs egyértelmű mozgás
        (0, 0)
    }
}
"""

prog = prog.replace("    fn apply_step(graph: &KernelStructureGraph", gravitate_fn + "\n    fn apply_step(graph: &KernelStructureGraph")

# apply_step: SemanticGravitate kezelése
old_gravitate_match = """                Transformation::SemanticRecolorToTarget => {
                    if let Some(spec) = &step.target_spec {
                        if let Some((_, _, Some(color))) = Self::resolve_target_spec(spec, graph, gw, gh) {
                            let recolor = Transformation::Recolor { node_id: node.id.clone(), new_color: color };
                            result = crate::sandbox::operators::apply_transformation(&result, &recolor);
                        }
                    }
                }"""
new_gravitate_match = """                Transformation::SemanticGravitate => {
                    if let Some(spec) = &step.target_spec {
                        if let Some((ax, ay, _)) = Self::resolve_target_spec(spec, graph, gw, gh) {
                            // Keressük meg a horgony objektumot
                            if let Some(anchor_node) = graph.nodes.iter().find(|n| {
                                let nx: i64 = n.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                                let ny: i64 = n.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                                nx == ax && ny == ay
                            }) {
                                let (dx, dy) = gravitate(&node, anchor_node);
                                let translate = Transformation::Translate { node_id: node.id.clone(), dx, dy };
                                result = crate::sandbox::operators::apply_transformation(&result, &translate);
                            }
                        }
                    }
                }
                Transformation::SemanticRecolorToTarget => {
                    if let Some(spec) = &step.target_spec {
                        if let Some((_, _, Some(color))) = Self::resolve_target_spec(spec, graph, gw, gh) {
                            let recolor = Transformation::Recolor { node_id: node.id.clone(), new_color: color };
                            result = crate::sandbox::operators::apply_transformation(&result, &recolor);
                        }
                    }
                }"""
prog = prog.replace(old_gravitate_match, new_gravitate_match)

with open(prog_path, 'w') as f:
    f.write(prog)
print("gravitate() function and apply_step extended for SemanticGravitate")
PYEOF

# ===== 4. step_signature frissítése a generator.rs-ben =====
python3 << 'PYEOF'
gen_path = "src/semantic_hypothesis/generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

# step_signature: hozzáadjuk a GravitateAnchor kezelését
old_sig = """        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
    };"""
new_sig = """        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
        Some(TargetSpec::GravitateAnchor { anchor_predicate }) => format!("GravitateAnchor:{}", anchor_predicate.name()),
    };"""
gen = gen.replace(old_sig, new_sig)

with open(gen_path, 'w') as f:
    f.write(gen)
print("step_signature extended for GravitateAnchor")
PYEOF

# ===== 5. Build & teszt =====
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -20
echo "===== COVERAGE 05f2a901 ====="
target/release/arc_abstraction_coverage ARC-AGI-master/data/training/05f2a901.json 2>&1 | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'Programs: {len(d[\"programs\"])}, Best coverage: {d[\"best_coverage\"]*100:.1f}%')"
echo "===== COMMIT ====="
git add -A && git commit -m "feat: add Gravitate primitive (TargetSpec::GravitateAnchor, Transformation::SemanticGravitate)" && git push
