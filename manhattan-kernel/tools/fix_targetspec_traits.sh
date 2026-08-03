#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. TargetSpec kézi trait implementációk és hiányzó match ágak
python3 << 'PYEOF'
prog_path = "src/abstraction/program.rs"
with open(prog_path, 'r') as f:
    prog = f.read()

# Eltávolítjuk a PartialEq-et a TargetSpec derive-jából (mostantól kézzel)
prog = prog.replace("#[derive(Debug, Clone, PartialEq)]\npub enum TargetSpec {", "#[derive(Debug, Clone)]\npub enum TargetSpec {")

# Hozzáadjuk a kézi PartialEq implementációt a TargetSpec után
partialeq_impl = """
impl PartialEq for TargetSpec {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TargetSpec::Constant(a), TargetSpec::Constant(b)) => a == b,
            (TargetSpec::RelativeToNode { condition: c1, dx_offset: dx1, dy_offset: dy1 },
             TargetSpec::RelativeToNode { condition: c2, dx_offset: dx2, dy_offset: dy2 }) =>
                c1 == c2 && dx1 == dx2 && dy1 == dy2,
            (TargetSpec::GridAnchor { corner: c1 }, TargetSpec::GridAnchor { corner: c2 }) => c1 == c2,
            (TargetSpec::CopyAttributeFrom { condition: c1, attribute: a1 },
             TargetSpec::CopyAttributeFrom { condition: c2, attribute: a2 }) => c1 == c2 && a1 == a2,
            (TargetSpec::SemanticRelation { relation: r1, anchor_predicate: p1 },
             TargetSpec::SemanticRelation { relation: r2, anchor_predicate: p2 }) =>
                r1 == r2 && p1.name() == p2.name(),
            _ => false,
        }
    }
}
"""

# Beszúrjuk a TargetSpec blokk után (a GridCorner előtt)
prog = prog.replace("#[derive(Debug, Clone, PartialEq)]\npub enum GridCorner", partialeq_impl + "\n#[derive(Debug, Clone, PartialEq)]\npub enum GridCorner")

# step_signature a generator.rs-ben: hozzáadjuk a SemanticRelation ágat
gen_path = "src/semantic_hypothesis/generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

old_sig = """        Some(TargetSpec::SemanticRelation { relation, .. }) => format!("SemanticRelation:{:?}", relation),
    }"""
new_sig = """        Some(TargetSpec::SemanticRelation { relation, .. }) => format!("SemanticRelation:{:?}", relation),
        _ => "Unknown".to_string(),
    }"""
gen = gen.replace(old_sig, new_sig)

with open(gen_path, 'w') as f:
    f.write(gen)

with open(prog_path, 'w') as f:
    f.write(prog)
print("TargetSpec traits and generator signature fixed.")
PYEOF

# 2. Build
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -15
echo "===== COMMIT ====="
git add -A && git commit -m "fix: manual PartialEq for TargetSpec, add missing SemanticRelation match arm" && git push
