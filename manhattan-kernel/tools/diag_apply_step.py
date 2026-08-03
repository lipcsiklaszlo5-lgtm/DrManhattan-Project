import pathlib, subprocess

ROOT = pathlib.Path("/workspaces/DrManhattan-Project/manhattan-kernel")

def run(cmd):
    print(f">>> {cmd}", flush=True)
    p = subprocess.run(cmd, shell=True, cwd=str(ROOT), capture_output=True, text=True)
    print(p.stdout)
    if p.stderr:
        print(p.stderr)

# 1. program.rs: ideiglenes diagnosztikai log az apply_step-be
prog_path = ROOT / "src" / "abstraction" / "program.rs"
with open(prog_path, 'r') as f:
    prog = f.read()

# Beszúrás a for node in candidates ciklus elejére
old_candidates = "        for node in candidates {"
new_candidates = """        eprintln!("[APPLY] === pair start ===");
        for node in candidates {
            eprintln!("[APPLY]   selected object: {}", node.id);
            eprintln!("[APPLY]   selected bbox: x={}, y={}, w={}, h={}",
                node.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0),
                node.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0),
                node.attributes.get("bbox_w").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0),
                node.attributes.get("bbox_h").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0)
            );
            if let Some(spec) = &step.target_spec {
                eprintln!("[APPLY]   target_spec={:?}", spec);
                if let Some((tx, ty, color_opt)) = Self::resolve_target_spec(spec, graph, gw, gh) {
                    eprintln!("[APPLY]   computed target: ({}, {})", tx, ty);
                }
            }"""

prog = prog.replace(old_candidates, new_candidates)

# Beszúrás a translate alkalmazása után
old_translate = """                            let translate = Transformation::Translate { node_id: node.id.clone(), dx, dy };
                            result = crate::sandbox::operators::apply_transformation(&result, &translate);"""
new_translate = """                            eprintln!("[APPLY]   translation: dx={}, dy={}", dx, dy);
                            let translate = Transformation::Translate { node_id: node.id.clone(), dx, dy };
                            result = crate::sandbox::operators::apply_transformation(&result, &translate);
                            // Log predicted bbox after translation
                            if let Some(pred_node) = result.nodes.iter().find(|n| n.id == node.id) {
                                eprintln!("[APPLY]   predicted bbox: x={}, y={}, w={}, h={}",
                                    pred_node.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0),
                                    pred_node.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0),
                                    pred_node.attributes.get("bbox_w").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0),
                                    pred_node.attributes.get("bbox_h").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0)
                                );
                            }"""

prog = prog.replace(old_translate, new_translate)

prog_path.write_text(prog)
print("Diagnostic logging inserted into apply_step")

# 2. Build & run on 05f2a901
run("cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -5")
print("\n===== DIAGNOSTIC OUTPUT =====")
run("MK_DIAG=1 target/release/arc_abstraction_coverage ARC-AGI-master/data/training/05f2a901.json 2>&1 | grep -A5 'APPLY'")

# 3. Restore program.rs
run("git checkout -- src/abstraction/program.rs")
print("program.rs restored to original state")
