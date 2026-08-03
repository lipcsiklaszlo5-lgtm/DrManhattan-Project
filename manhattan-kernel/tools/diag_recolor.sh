#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. Diagnosztikai üzenet beszúrása az apply_step SemanticRecolorToTarget ágába
python3 << 'PYEOF'
prog_path = "/workspaces/DrManhattan-Project/manhattan-kernel/src/abstraction/program.rs"
with open(prog_path, 'r') as f:
    prog = f.read()

old_recolor = """                Transformation::SemanticRecolorToTarget => {
                    if let Some(spec) = &step.target_spec {
                        if let Some((_, _, Some(color))) = Self::resolve_target_spec(spec, graph, gw, gh) {
                            let recolor = Transformation::Recolor { node_id: node.id.clone(), new_color: color };
                            result = crate::sandbox::operators::apply_transformation(&result, &recolor);
                        }
                    }
                }"""

new_recolor = """                Transformation::SemanticRecolorToTarget => {
                    eprintln!("[DIAG] SemanticRecolorToTarget: target_spec={:?}", step.target_spec);
                    if let Some(spec) = &step.target_spec {
                        let resolved = Self::resolve_target_spec(spec, graph, gw, gh);
                        eprintln!("[DIAG] resolve_target_spec returned: {:?}", resolved);
                        if let Some((_, _, Some(color))) = resolved {
                            eprintln!("[DIAG] applying recolor with color={}", color);
                            let recolor = Transformation::Recolor { node_id: node.id.clone(), new_color: color };
                            result = crate::sandbox::operators::apply_transformation(&result, &recolor);
                        } else {
                            eprintln!("[DIAG] resolve_target_spec did NOT return a color");
                        }
                    } else {
                        eprintln!("[DIAG] no target_spec – skipping recolor");
                    }
                }"""

prog = prog.replace(old_recolor, new_recolor)
with open(prog_path, 'w') as f:
    f.write(prog)
print("Diagnostic inserted into SemanticRecolorToTarget")
PYEOF

# 2. Build & run on 017c7c7b (MK_DIAG=1 a részletes logokért)
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -5
echo "===== DIAGNOSTIC RUN ====="
MK_DIAG=1 target/release/arc_abstraction_coverage ARC-AGI-master/data/training/017c7c7b.json 2>&1 | head -100

# 3. Visszaállítás
echo "===== RESTORE ====="
git checkout -- src/abstraction/program.rs
