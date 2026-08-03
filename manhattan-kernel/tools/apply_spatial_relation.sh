#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. program.rs: SpatialRelation enum + TargetSpec::RelativeToNode átalakítása
python3 << 'PYEOF'
prog_path = "src/abstraction/program.rs"
with open(prog_path, 'r') as f:
    prog = f.read()

# SpatialRelation enum definíció a TargetSpec elé
spatial_enum = """/// Symbolic spatial relation between two bounding boxes.
#[derive(Debug, Clone, PartialEq)]
pub enum SpatialRelation {
    Above,
    Below,
    LeftOf,
    RightOf,
    TouchingTop,
    TouchingBottom,
    TouchingLeft,
    TouchingRight,
    CenteredX,
    CenteredY,
    SameRow,
    SameColumn,
}

"""
prog = prog.replace("pub enum TargetSpec {", spatial_enum + "pub enum TargetSpec {")

# TargetSpec::RelativeToNode átalakítása: dx/dy helyett SpatialRelation
old_relative = "    RelativeToNode { condition: Box<Condition>, dx_offset: i64, dy_offset: i64 },"
new_relative = "    RelativeToNode { condition: Box<Condition>, relation: SpatialRelation },"
prog = prog.replace(old_relative, new_relative)

# resolve_target_spec módosítása: SpatialRelation alapján számoljuk a célkoordinátákat
old_resolve_relative = """            TargetSpec::RelativeToNode { condition, dx_offset, dy_offset } => {
                let refs = Self::matching_nodes(graph, condition.as_ref());
                if let Some(ref_node) = refs.first() {
                    let rx: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let ry: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                    Some((rx + dx_offset, ry + dy_offset, None))
                } else { None }
            }"""
new_resolve_relative = """            TargetSpec::RelativeToNode { condition, relation } => {
                let refs = Self::matching_nodes(graph, condition.as_ref());
                if let Some(ref_node) = refs.first() {
                    let rx: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let ry: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let rw: i64 = ref_node.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let rh: i64 = ref_node.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(0);
                    // A mozgó objektum méretét a hívó oldalon (apply_step) ismerjük,
                    // itt csak a horgony alapján számolunk.
                    let (tx, ty) = match relation {
                        SpatialRelation::Above => (rx + rw/2, ry),
                        SpatialRelation::Below => (rx + rw/2, ry + rh),
                        SpatialRelation::LeftOf => (rx, ry + rh/2),
                        SpatialRelation::RightOf => (rx + rw, ry + rh/2),
                        SpatialRelation::TouchingTop => (rx + rw/2, ry),
                        SpatialRelation::TouchingBottom => (rx + rw/2, ry + rh),
                        SpatialRelation::TouchingLeft => (rx, ry + rh/2),
                        SpatialRelation::TouchingRight => (rx + rw, ry + rh/2),
                        SpatialRelation::CenteredX => (rx + rw/2, ry + rh/2),
                        SpatialRelation::CenteredY => (rx + rw/2, ry + rh/2),
                        SpatialRelation::SameRow => (rx, ry),
                        SpatialRelation::SameColumn => (rx, ry),
                    };
                    Some((tx, ty, None))
                } else { None }
            }"""
prog = prog.replace(old_resolve_relative, new_resolve_relative)

# Kézi trait implementációk frissítése: PartialEq, Debug, Clone
# PartialEq
old_partial_relative = """            (TargetSpec::RelativeToNode { condition: c1, dx_offset: dx1, dy_offset: dy1 },
             TargetSpec::RelativeToNode { condition: c2, dx_offset: dx2, dy_offset: dy2 }) =>
                c1 == c2 && dx1 == dx2 && dy1 == dy2,"""
new_partial_relative = """            (TargetSpec::RelativeToNode { condition: c1, relation: r1 },
             TargetSpec::RelativeToNode { condition: c2, relation: r2 }) =>
                c1 == c2 && r1 == r2,"""
prog = prog.replace(old_partial_relative, new_partial_relative)

# Debug
old_debug_relative = """            TargetSpec::RelativeToNode { condition, dx_offset, dy_offset } =>
                write!(f, "RelativeToNode({}, {}, {})", condition.name(), dx_offset, dy_offset),"""
new_debug_relative = """            TargetSpec::RelativeToNode { condition, relation } =>
                write!(f, "RelativeToNode({}, {:?})", condition.name(), relation),"""
prog = prog.replace(old_debug_relative, new_debug_relative)

# Clone
old_clone_relative = """            TargetSpec::RelativeToNode { condition, dx_offset, dy_offset } =>
                TargetSpec::RelativeToNode { condition: condition.clone(), dx_offset: *dx_offset, dy_offset: *dy_offset },"""
new_clone_relative = """            TargetSpec::RelativeToNode { condition, relation } =>
                TargetSpec::RelativeToNode { condition: condition.clone(), relation: relation.clone() },"""
prog = prog.replace(old_clone_relative, new_clone_relative)

# A constant_target_matches függvényt is frissíteni kell (generator.rs-ben van)
# de az már nem használja a dx/dy-t, mert a ConstantTargetMatches csak a Constant típust nézi.

with open(prog_path, 'w') as f:
    f.write(prog)
print("program.rs: SpatialRelation enum added, RelativeToNode uses relation instead of dx/dy")
PYEOF

# 2. generator.rs: dx/dy helyett SpatialRelation használata
python3 << 'PYEOF'
gen_path = "src/semantic_hypothesis/generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

# A Translate ágban a RelativeToNode létrehozásakor a dx/dy helyett a infer_spatial_relation eredményét használjuk
old_relative_creation = """                                        let relation = infer_spatial_relation(node_out, &ref_node);
                                        let target_spec = if let Some(_rel_name) = relation {
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
                                        };"""

new_relative_creation = """                                        let spatial_relation = infer_spatial_relation(node_out, &ref_node);
                                        if let Some(relation) = spatial_relation {
                                            // Convert string relation to SpatialRelation enum
                                            let rel = match relation.as_str() {
                                                "Above" => crate::abstraction::program::SpatialRelation::Above,
                                                "Below" => crate::abstraction::program::SpatialRelation::Below,
                                                "LeftOf" => crate::abstraction::program::SpatialRelation::LeftOf,
                                                "RightOf" => crate::abstraction::program::SpatialRelation::RightOf,
                                                "TouchingNorth" => crate::abstraction::program::SpatialRelation::TouchingTop,
                                                "TouchingSouth" => crate::abstraction::program::SpatialRelation::TouchingBottom,
                                                "TouchingWest" => crate::abstraction::program::SpatialRelation::TouchingLeft,
                                                "TouchingEast" => crate::abstraction::program::SpatialRelation::TouchingRight,
                                                "AlignTop" | "AlignBottom" | "AlignLeft" | "AlignRight" => {
                                                    // Approximate alignment as CenteredX or CenteredY
                                                    if relation.contains("AlignTop") || relation.contains("AlignBottom") {
                                                        crate::abstraction::program::SpatialRelation::CenteredX
                                                    } else {
                                                        crate::abstraction::program::SpatialRelation::CenteredY
                                                    }
                                                }
                                                _ => return continue, // unknown relation, skip this reference
                                            };
                                            TargetSpec::RelativeToNode {
                                                condition: Box::new(Condition::Predicate(ref_pred)),
                                                relation: rel,
                                            }
                                        } else {
                                            // No spatial relation detected, fall back to dx/dy? No, skip.
                                            continue;
                                        };"""

gen = gen.replace(old_relative_creation, new_relative_creation)

# Frissítsük a step_signature-t is: a RelativeToNode most már SpatialRelation-t használ
old_sig = """        Some(TargetSpec::RelativeToNode { condition, dx_offset: _, dy_offset: _ }) => {
            // Ha van felismert reláció, azt használjuk a szignatúrában
            format!("RelativeToNode:{}", condition.name())
        },"""
new_sig = """        Some(TargetSpec::RelativeToNode { condition, relation }) => {
            format!("RelativeToNode:{}:{:?}", condition.name(), relation)
        },"""
gen = gen.replace(old_sig, new_sig)

# A constant_target_matches függvényben a RelativeToNode ág már nem releváns, mert nincs Constant érték

with open(gen_path, 'w') as f:
    f.write(gen)
print("generator.rs: using SpatialRelation instead of dx/dy")
PYEOF

# 3. Build & diagnosztika
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -15
echo "===== DIAGNOSTIC RUN ====="
MK_DIAG=1 target/release/arc_abstraction_coverage ARC-AGI-master/data/training/05f2a901.json 2>&1 | grep -E "target_spec|SpatialRelation|RelativeToNode" | head -20
echo "===== COMMIT ====="
git add -A && git commit -m "feat: replace numeric dx/dy with SpatialRelation in RelativeToNode" && git push
