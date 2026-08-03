#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. Visszaállítás a Gravitate előtti stabil program.rs-re (bce0bac)
git checkout bce0bac -- src/abstraction/program.rs

# 2. TargetSpec bővítése a GravitateAnchor variánssal
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

# Kézi trait implementációk (Debug, Clone, PartialEq) frissítése
# PartialEq: hozzáadjuk a GravitateAnchor ágat
old_partial = """            (TargetSpec::CopyAttributeFrom { condition: c1, attribute: a1 },
             TargetSpec::CopyAttributeFrom { condition: c2, attribute: a2 }) => c1 == c2 && a1 == a2,
            _ => false,"""
new_partial = """            (TargetSpec::CopyAttributeFrom { condition: c1, attribute: a1 },
             TargetSpec::CopyAttributeFrom { condition: c2, attribute: a2 }) => c1 == c2 && a1 == a2,
            (TargetSpec::GravitateAnchor { anchor_predicate: p1 },
             TargetSpec::GravitateAnchor { anchor_predicate: p2 }) => p1.name() == p2.name(),
            _ => false,"""
prog = prog.replace(old_partial, new_partial)

# Debug: hozzáadjuk a GravitateAnchor ágat
old_debug = """            TargetSpec::CopyAttributeFrom { condition, attribute } =>
                write!(f, "CopyAttributeFrom({}, {})", condition.name(), attribute),
        }"""
new_debug = """            TargetSpec::CopyAttributeFrom { condition, attribute } =>
                write!(f, "CopyAttributeFrom({}, {})", condition.name(), attribute),
            TargetSpec::GravitateAnchor { anchor_predicate } =>
                write!(f, "GravitateAnchor({})", anchor_predicate.name()),
        }"""
prog = prog.replace(old_debug, new_debug)

# Clone: hozzáadjuk a GravitateAnchor ágat
old_clone = """            TargetSpec::CopyAttributeFrom { condition, attribute } =>
                TargetSpec::CopyAttributeFrom { condition: condition.clone(), attribute: attribute.clone() },
        }"""
new_clone = """            TargetSpec::CopyAttributeFrom { condition, attribute } =>
                TargetSpec::CopyAttributeFrom { condition: condition.clone(), attribute: attribute.clone() },
            TargetSpec::GravitateAnchor { anchor_predicate } =>
                TargetSpec::GravitateAnchor { anchor_predicate: anchor_predicate.clone_box() },
        }"""
prog = prog.replace(old_clone, new_clone)

# resolve_target_spec kiterjesztése
old_resolve = """            TargetSpec::CopyAttributeFrom { condition, attribute } => {
                let refs = Self::matching_nodes(graph, condition.as_ref());
                if let Some(ref_node) = refs.first() {
                    let val = ref_node.attributes.get(attribute).cloned();
                    Some((0, 0, val))
                } else { None }
            }

        }"""
new_resolve = """            TargetSpec::CopyAttributeFrom { condition, attribute } => {
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
prog = prog.replace(old_resolve, new_resolve)

# gravitate függvény és apply_step bővítése
gravitate_and_step = r"""
/// Calculate the translation needed for `moving` to touch `anchor`.
fn gravitate(moving: &crate::structure::Node, anchor: &crate::structure::Node) -> (i64, i64) {
    let mx: i64 = moving.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
    let my: i64 = moving.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
    let mw: i64 = moving.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1);
    let mh: i64 = moving.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1);
    let ax: i64 = anchor.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
    let ay: i64 = anchor.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
    let aw: i64 = anchor.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1);
    let ah: i64 = anchor.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1);

    let col_overlap = mx < ax + aw && mx + mw > ax;
    let row_overlap = my < ay + ah && my + mh > ay;

    if col_overlap && !row_overlap {
        let dy = if my + mh <= ay {
            ay - (my + mh)
        } else {
            ay + ah - my
        };
        (0, dy)
    } else if row_overlap && !col_overlap {
        let dx = if mx + mw <= ax {
            ax - (mx + mw)
        } else {
            ax + aw - mx
        };
        (dx, 0)
    } else {
        (0, 0)
    }
}

    fn apply_step(graph: &KernelStructureGraph, step: &AbstractStep, gw: u8, gh: u8) -> KernelStructureGraph {
"""

# Kicseréljük a régi apply_step fejlécet az újra
old_step_header = "    fn apply_step(graph: &KernelStructureGraph, step: &AbstractStep, gw: u8, gh: u8) -> KernelStructureGraph {"
prog = prog.replace(old_step_header, gravitate_and_step)

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
print("program.rs: GravitateAnchor re-added with complete traits and gravitate function")
PYEOF

# 3. Build & teszt
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -15
echo "===== COVERAGE 05f2a901 ====="
target/release/arc_abstraction_coverage ARC-AGI-master/data/training/05f2a901.json 2>&1 | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'Programs: {len(d[\"programs\"])}, Best coverage: {d[\"best_coverage\"]*100:.1f}%')"
echo "===== COMMIT ====="
git add -A && git commit -m "fix: rebuild Gravitate primitive with complete trait implementations" && git push
