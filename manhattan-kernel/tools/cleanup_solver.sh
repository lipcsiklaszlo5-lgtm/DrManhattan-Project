#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. Solver fájl törlése
rm -f src/solver.rs

# 2. lib.rs tisztítása
python3 << 'PYEOF'
with open("src/lib.rs", 'r') as f:
    lib = f.read()
lib = lib.replace("pub mod solver;\n", "")
with open("src/lib.rs", 'w') as f:
    f.write(lib)
print("lib.rs cleaned")
PYEOF

# 3. meta_learner.rs visszaállítása a finalize javítás utáni verzióra
git checkout 04b52a4 -- src/meta_learner.rs

# 4. Build & teszt
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -5
echo "===== COVERAGE 05f2a901 ====="
target/release/arc_abstraction_coverage ARC-AGI-master/data/training/05f2a901.json 2>&1 | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'Programs: {len(d[\"programs\"])}, Best coverage: {d[\"best_coverage\"]*100:.1f}%')"
echo "===== COMMIT ====="
git add -A && git commit -m "fix: remove broken Solver module, restore stable finalize" && git push
