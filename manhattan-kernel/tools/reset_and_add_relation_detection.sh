#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. program.rs visszaállítása a 6954786 commitra (SHE működő állapot)
git checkout 6954786 -- src/abstraction/program.rs

# 2. generator.rs step_signature visszaállítása
python3 << 'PYEOF'
gen_path = "src/semantic_hypothesis/generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()
# Visszaállítjuk a step_signature-t az eredeti verzióra
old_sig = """        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
        Some(TargetSpec::SemanticRelation { relation, .. }) => format!("SemanticRelation:{:?}", relation),
    };"""
new_sig = """        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
    };"""
gen = gen.replace(old_sig, new_sig)
with open(gen_path, 'w') as f:
    f.write(gen)
print("generator.rs step_signature restored")
PYEOF

# 3. Build ellenőrzés
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -5
echo "===== COVERAGE 05f2a901 ====="
target/release/arc_abstraction_coverage ARC-AGI-master/data/training/05f2a901.json 2>&1 | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'Programs: {len(d[\"programs\"])}, Best coverage: {d[\"best_coverage\"]*100:.1f}%')"
echo "===== COMMIT ====="
git add -A && git commit -m "fix: revert program.rs to stable, prepare for in-generator relation detection" && git push
