#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. generator.rs: Condition importálása
python3 << 'PYEOF'
gen_path = "/workspaces/DrManhattan-Project/manhattan-kernel/src/semantic_hypothesis/generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()
# Hozzáadjuk az importot
gen = gen.replace("use crate::abstraction::program::{TargetSpec, GridCorner};",
                  "use crate::abstraction::program::{TargetSpec, GridCorner};\nuse crate::abstraction::transform::Condition;")
with open(gen_path, 'w') as f:
    f.write(gen)
print("Condition import added to generator.rs")
PYEOF

# 2. transform.rs: Clone kézi implementálása a derive helyett
python3 << 'PYEOF'
trans_path = "/workspaces/DrManhattan-Project/manhattan-kernel/src/abstraction/transform.rs"
with open(trans_path, 'r') as f:
    trans = f.read()

# Eltávolítjuk a #[derive(Clone)]-t és kézzel implementáljuk a Clone-t
trans = trans.replace("#[derive(Clone)]\npub enum Condition {",
                      "pub enum Condition {")

# Hozzáadjuk a kézi Clone implementációt a Condition után (az impl Debug előtt)
clone_impl = """
impl Clone for Condition {
    fn clone(&self) -> Self {
        match self {
            Condition::AlwaysTrue => Condition::AlwaysTrue,
            Condition::NodeHasAttribute(a, v) => Condition::NodeHasAttribute(a.clone(), v.clone()),
            Condition::ColorEquals(c) => Condition::ColorEquals(c.clone()),
            Condition::PositionAbove(id) => Condition::PositionAbove(id.clone()),
            Condition::PositionLeftOf(id) => Condition::PositionLeftOf(id.clone()),
            Condition::Unique(s) => Condition::Unique(s.clone()),
            Condition::ExtremeByAttribute { attribute, mode } => Condition::ExtremeByAttribute { attribute: attribute.clone(), mode: mode.clone() },
            Condition::TouchesBorder => Condition::TouchesBorder,
            Condition::And(conds) => Condition::And(conds.clone()),
            Condition::Not(cond) => Condition::Not(cond.clone()),
            Condition::StructuralRole(r) => Condition::StructuralRole(*r),
            Condition::Predicate(p) => Condition::Predicate(p.clone_box()),
        }
    }
}
"""

trans = trans.replace("impl std::fmt::Debug for Condition {", clone_impl + "\nimpl std::fmt::Debug for Condition {")

with open(trans_path, 'w') as f:
    f.write(trans)
print("Manual Clone implemented for Condition")
PYEOF

# 3. Build & test
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -10
echo "===== COVERAGE 017c7c7b ====="
target/release/arc_abstraction_coverage ARC-AGI-master/data/training/017c7c7b.json 2>&1
echo "===== COMMIT ====="
git add -A && git commit -m "fix: import Condition in generator.rs, manual Clone for Condition" && git push
