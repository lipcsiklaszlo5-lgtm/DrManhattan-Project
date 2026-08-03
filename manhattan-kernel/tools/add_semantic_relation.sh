#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# ===== 1. program.rs: SpatialRelation enum + TargetSpec::SemanticRelation =====
python3 << 'PYEOF'
prog_path = "src/abstraction/program.rs"
with open(prog_path, 'r') as f:
    prog = f.read()

# SpatialRelation enum a TargetSpec előtt
spatial_enum = """/// Spatial relation between two objects (anchor → target)
#[derive(Debug, Clone, PartialEq)]
pub enum SpatialRelation {
    Above,
    Below,
    LeftOf,
    RightOf,
    TouchingNorth,
    TouchingSouth,
    TouchingEast,
    TouchingWest,
    AlignTop,
    AlignBottom,
    AlignLeft,
    AlignRight,
    CenterInside,
    MirrorHorizontal,
    MirrorVertical,
    RotateAround,
}

"""

# Beszúrjuk a TargetSpec elé
prog = prog.replace("pub enum TargetSpec {", spatial_enum + "pub enum TargetSpec {")

# Új variáns a TargetSpec-ben
old_enum = """    CopyAttributeFrom { condition: Box<Condition>, attribute: String },
}"""
new_enum = """    CopyAttributeFrom { condition: Box<Condition>, attribute: String },
    /// Semantic spatial relation between moved object and anchor
    SemanticRelation { relation: SpatialRelation, anchor_predicate: Box<dyn Predicate> },
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
            TargetSpec::SemanticRelation { relation, anchor_predicate } => {
                let refs = Self::matching_nodes(graph, anchor_predicate.as_ref());
                if let Some(ref_node) = refs.first() {
                    let rx: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let ry: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let rw: i64 = ref_node.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let rh: i64 = ref_node.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(0);
                    // A mozgó objektum méreteit is kiolvassuk a kiválasztott node-ból (a hívó oldalon)
                    // Itt a célkoordinátákat a reláció alapján számoljuk
                    let (tx, ty) = match relation {
                        SpatialRelation::Above => (rx + rw/2, ry - rh/2),
                        SpatialRelation::Below => (rx + rw/2, ry + rh + rh/2),
                        SpatialRelation::LeftOf => (rx - rw/2, ry + rh/2),
                        SpatialRelation::RightOf => (rx + rw + rw/2, ry + rh/2),
                        SpatialRelation::TouchingNorth => (rx + rw/2, ry),
                        SpatialRelation::TouchingSouth => (rx + rw/2, ry + rh),
                        SpatialRelation::TouchingEast => (rx + rw, ry + rh/2),
                        SpatialRelation::TouchingWest => (rx, ry + rh/2),
                        SpatialRelation::AlignTop => (rx + rw/2, ry),
                        SpatialRelation::AlignBottom => (rx + rw/2, ry + rh),
                        SpatialRelation::AlignLeft => (rx, ry + rh/2),
                        SpatialRelation::AlignRight => (rx + rw, ry + rh/2),
                        SpatialRelation::CenterInside => (rx + rw/2, ry + rh/2),
                        SpatialRelation::MirrorHorizontal => (grid_width as i64 - rx - rw, ry),
                        SpatialRelation::MirrorVertical => (rx, grid_height as i64 - ry - rh),
                        SpatialRelation::RotateAround => (rx + rw/2, ry + rh/2), // placeholder
                    };
                    Some((tx, ty, None))
                } else { None }
            }

        }"""
prog = prog.replace(old_resolve_end, new_resolve_end)

with open(prog_path, 'w') as f:
    f.write(prog)
print("program.rs: SpatialRelation + SemanticRelation added, resolve_target_spec extended")
PYEOF

# ===== 2. generator.rs: step_signature frissítése =====
python3 << 'PYEOF'
gen_path = "src/semantic_hypothesis/generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

# step_signature: SemanticRelation kezelése
old_sig = """        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
    }"""
new_sig = """        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
        Some(TargetSpec::SemanticRelation { relation, .. }) => format!("SemanticRelation:{:?}", relation),
    }"""
gen = gen.replace(old_sig, new_sig)

with open(gen_path, 'w') as f:
    f.write(gen)
print("generator.rs: step_signature updated for SemanticRelation")
PYEOF

# ===== 3. Build =====
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -15
echo "===== COMMIT ====="
git add -A && git commit -m "feat: add SpatialRelation enum and TargetSpec::SemanticRelation" && git push
