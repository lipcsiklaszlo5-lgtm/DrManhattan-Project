#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# Javítás: a Condition::name() adjon egyedi nevet a Predicate variánsra
python3 << 'PYEOF'
file = "/workspaces/DrManhattan-Project/manhattan-kernel/src/abstraction/transform.rs"
with open(file, 'r') as f:
    content = f.read()

# Kicseréljük a name() metódust, hogy a Predicate esetén a belső predikátum nevét adja vissza
old_name = '    fn name(&self) -> &str { "Condition" }'
new_name = '''    fn name(&self) -> &str {
        match self {
            Condition::Predicate(p) => p.name(),
            _ => "Condition",
        }
    }'''

content = content.replace(old_name, new_name)
with open(file, 'w') as f:
    f.write(content)
print("Condition::name() mostantól delegál a belső predikátumra.")
PYEOF

# Build & test
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -3
echo "===== COVERAGE 05f2a901 ====="
MK_DIAG=1 target/release/arc_abstraction_coverage ARC-AGI-master/data/training/05f2a901.json 2>&1 | grep -E "ACCEPTED|returning|coverage"
echo "===== COMMIT ====="
git add -A && git commit -m "fix: delegate Condition::name() to inner predicate for unique signatures" && git push
