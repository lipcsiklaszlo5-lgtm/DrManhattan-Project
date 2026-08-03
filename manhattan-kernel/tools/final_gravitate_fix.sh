#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. Visszaállítás a stabil bce0bac commitra (generator.rs)
git checkout bce0bac -- src/semantic_hypothesis/generator.rs

# 2. program.rs: TargetSpec trait implementációk javítása
python3 << 'PYEOF'
prog_path = "src/abstraction/program.rs"
with open(prog_path, 'r') as f:
    prog = f.read()

# Hozzáadjuk a GravitateAnchor kezelését a kézi PartialEq implementációhoz
old_partialeq = """            (TargetSpec::CopyAttributeFrom { condition: c1, attribute: a1 },
             TargetSpec::CopyAttributeFrom { condition: c2, attribute: a2 }) => c1 == c2 && a1 == a2,
            _ => false,"""
new_partialeq = """            (TargetSpec::CopyAttributeFrom { condition: c1, attribute: a1 },
             TargetSpec::CopyAttributeFrom { condition: c2, attribute: a2 }) => c1 == c2 && a1 == a2,
            (TargetSpec::GravitateAnchor { anchor_predicate: p1 },
             TargetSpec::GravitateAnchor { anchor_predicate: p2 }) => p1.name() == p2.name(),
            _ => false,"""
prog = prog.replace(old_partialeq, new_partialeq)

# Hozzáadjuk a GravitateAnchor kezelését a kézi Debug implementációhoz
old_debug = """            TargetSpec::CopyAttributeFrom { condition, attribute } =>
                write!(f, "CopyAttributeFrom({}, {})", condition.name(), attribute),
        }"""
new_debug = """            TargetSpec::CopyAttributeFrom { condition, attribute } =>
                write!(f, "CopyAttributeFrom({}, {})", condition.name(), attribute),
            TargetSpec::GravitateAnchor { anchor_predicate } =>
                write!(f, "GravitateAnchor({})", anchor_predicate.name()),
        }"""
prog = prog.replace(old_debug, new_debug)

# Hozzáadjuk a GravitateAnchor kezelését a kézi Clone implementációhoz
old_clone = """            TargetSpec::CopyAttributeFrom { condition, attribute } =>
                TargetSpec::CopyAttributeFrom { condition: condition.clone(), attribute: attribute.clone() },
        }"""
new_clone = """            TargetSpec::CopyAttributeFrom { condition, attribute } =>
                TargetSpec::CopyAttributeFrom { condition: condition.clone(), attribute: attribute.clone() },
            TargetSpec::GravitateAnchor { anchor_predicate } =>
                TargetSpec::GravitateAnchor { anchor_predicate: anchor_predicate.clone_box() },
        }"""
prog = prog.replace(old_clone, new_clone)

with open(prog_path, 'w') as f:
    f.write(prog)
print("program.rs: TargetSpec traits fixed for GravitateAnchor")
PYEOF

# 3. generator.rs: step_signature kiegészítése
python3 << 'PYEOF'
gen_path = "src/semantic_hypothesis/generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

old_match = """    let target_kind = match &step.target_spec {
        None => "None".to_string(),
        Some(TargetSpec::Constant(_)) => "Constant".to_string(),
        Some(TargetSpec::GridAnchor { corner }) => format!("GridAnchor:{:?}", corner),
        Some(TargetSpec::RelativeToNode { .. }) => "RelativeToNode".to_string(),
        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
    };"""

new_match = """    let target_kind = match &step.target_spec {
        None => "None".to_string(),
        Some(TargetSpec::Constant(_)) => "Constant".to_string(),
        Some(TargetSpec::GridAnchor { corner }) => format!("GridAnchor:{:?}", corner),
        Some(TargetSpec::RelativeToNode { .. }) => "RelativeToNode".to_string(),
        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
        Some(TargetSpec::GravitateAnchor { anchor_predicate }) => format!("GravitateAnchor:{}", anchor_predicate.name()),
    };"""

gen = gen.replace(old_match, new_match)

with open(gen_path, 'w') as f:
    f.write(gen)
print("generator.rs: step_signature extended")
PYEOF

# 4. Build & teszt
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -10
echo "===== COVERAGE 05f2a901 ====="
target/release/arc_abstraction_coverage ARC-AGI-master/data/training/05f2a901.json 2>&1 | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'Programs: {len(d[\"programs\"])}, Best coverage: {d[\"best_coverage\"]*100:.1f}%')"
echo "===== COMMIT ====="
git add -A && git commit -m "fix: complete TargetSpec traits for GravitateAnchor, finalize step_signature" && git push
