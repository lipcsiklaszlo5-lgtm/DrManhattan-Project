#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -3

echo "===== COVERAGE 5 FELADATON ====="
for TASK in $(ls ARC-AGI-master/data/training/*.json | head -10); do
    TASKNAME=$(basename "$TASK")
    RESULT=$(target/release/arc_abstraction_coverage "$TASK" 2>/dev/null)
    COV=$(echo "$RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['best_coverage'])" 2>/dev/null || echo "error")
    PROGS=$(echo "$RESULT" | python3 -c "import sys,json; print(len(json.load(sys.stdin).get('programs',[])))" 2>/dev/null || echo "error")
    echo "$TASKNAME: coverage=$COV programs=$PROGS"
done
