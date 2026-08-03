#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# Diagnosztika: hány node-ot választ ki a feltétel?
python3 << 'PYEOF'
prog_path = "/workspaces/DrManhattan-Project/manhattan-kernel/src/abstraction/program.rs"
with open(prog_path, 'r') as f:
    prog = f.read()

# Beszúrunk egy diagnosztikai üzenetet a select_candidates után
old_apply = """        let mut result = graph.clone();
        for node in candidates {"""

new_apply = """        eprintln!("[DIAG] apply_step: {} candidates selected for condition {:?}", candidates.len(), step.condition.as_ref().map(|c| c.name()));
        let mut result = graph.clone();
        for node in candidates {"""

prog = prog.replace(old_apply, new_apply)
with open(prog_path, 'w') as f:
    f.write(prog)
print("Diagnostic inserted into apply_step")
PYEOF

echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -5
echo "===== DIAGNOSTIC RUN ====="
MK_DIAG=1 target/release/arc_abstraction_coverage ARC-AGI-master/data/training/017c7c7b.json 2>&1 | grep -E "candidates selected|DIAG" | head -20

echo "===== RESTORE ====="
git checkout -- src/abstraction/program.rs
