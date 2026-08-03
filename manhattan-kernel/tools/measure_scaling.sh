#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -3

echo "===== COVERAGE 20 RANDOM ARC TASKS ====="
# Véletlenszerű 20 feladat kiválasztása a training könyvtárból
TASKS=$(ls ARC-AGI-master/data/training/*.json | shuf | head -20)
SOLVED=0
TOTAL=0
for TASK in $TASKS; do
    TASKNAME=$(basename "$TASK")
    RESULT=$(target/release/arc_abstraction_coverage "$TASK" 2>/dev/null)
    COV=$(echo "$RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin)['best_coverage'])" 2>/dev/null || echo "0.0")
    if [ "$COV" != "0.0" ]; then
        echo "$TASKNAME: coverage=$COV ✅"
        SOLVED=$((SOLVED + 1))
    else
        echo "$TASKNAME: coverage=$COV"
    fi
    TOTAL=$((TOTAL + 1))
done

echo ""
echo "=============================="
echo "SUMMARY: $SOLVED / $TOTAL tasks have coverage > 0%"
echo "=============================="
