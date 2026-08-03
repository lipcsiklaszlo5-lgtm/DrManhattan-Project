#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. step_signature javítása wildcard ággal
python3 << 'PYEOF'
gen_path = "src/semantic_hypothesis/generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

# Kicseréljük a teljes match blokkot a step_signature-ben
old_match = """    let target_kind = match &step.target_spec {
        None => "None".to_string(),
        Some(TargetSpec::Constant(_)) => "Constant".to_string(),
        Some(TargetSpec::GridAnchor { corner }) => format!("GridAnchor:{:?}", corner),
        Some(TargetSpec::RelativeToNode { .. }) => "RelativeToNode".to_string(),
        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
        Some(TargetSpec::GravitateAnchor { anchor_predicate }) => format!("GravitateAnchor:{}", anchor_predicate.name()),
    };"""

new_match = """    let target_kind = match &step.target_spec {
        None => "None".to_string(),
        Some(TargetSpec::Constant(_)) => "Constant".to_string(),
        Some(TargetSpec::GridAnchor { corner }) => format!("GridAnchor:{:?}", corner),
        Some(TargetSpec::RelativeToNode { .. }) => "RelativeToNode".to_string(),
        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
        Some(TargetSpec::GravitateAnchor { anchor_predicate }) => format!("GravitateAnchor:{}", anchor_predicate.name()),
        _ => "Unknown".to_string(),
    };"""

gen = gen.replace(old_match, new_match)

with open(gen_path, 'w') as f:
    f.write(gen)
print("step_signature fixed with wildcard arm")
PYEOF

# 2. Build & teszt
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -10
echo "===== COVERAGE 05f2a901 ====="
target/release/arc_abstraction_coverage ARC-AGI-master/data/training/05f2a901.json 2>&1 | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'Programs: {len(d[\"programs\"])}, Best coverage: {d[\"best_coverage\"]*100:.1f}%')"
echo "===== COMMIT ====="
git add -A && git commit -m "fix: add wildcard arm to step_signature match" && git push
