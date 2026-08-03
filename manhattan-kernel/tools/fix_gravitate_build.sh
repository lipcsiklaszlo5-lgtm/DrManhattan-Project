#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. generator.rs visszaállítása a stabil verzióra (bce0bac commit)
git checkout bce0bac -- src/semantic_hypothesis/generator.rs

# 2. step_signature kiegészítése a GravitateAnchor variánssal
python3 << 'PYEOF'
gen_path = "src/semantic_hypothesis/generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

# Kiegészítjük a step_signature match-et
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

# 3. Build & teszt
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -10
echo "===== COVERAGE 05f2a901 ====="
target/release/arc_abstraction_coverage ARC-AGI-master/data/training/05f2a901.json 2>&1 | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'Programs: {len(d[\"programs\"])}, Best coverage: {d[\"best_coverage\"]*100:.1f}%')"
echo "===== COMMIT ====="
git add -A && git commit -m "fix: add GravitateAnchor to step_signature, stabilize generator.rs" && git push
