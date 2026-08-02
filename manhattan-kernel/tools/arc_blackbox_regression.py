#!/usr/bin/env python3
"""Black-Box ARC Regression Suite for Manhattan Kernel.

Uses only the public predict() API via a Rust binary.
Compares against golden baseline, checks determinism,
and produces a categorized report.
"""

import json
import subprocess
import sys
import os
from pathlib import Path
from datetime import datetime

PROJECT_ROOT = Path(__file__).resolve().parent.parent
BINARY_TARGET = "arc_regression_runner"
ARC_DATA_DIR = PROJECT_ROOT / "ARC-AGI-master" / "data" / "training"
GOLDEN_BASELINE = PROJECT_ROOT / "golden" / "baseline.json"
GOLDEN_OUTPUTS = PROJECT_ROOT / "golden" / "expected_outputs"

def run_binary(arc_dir: str) -> dict:
    """Compile and run the regression binary, return parsed JSON report."""
    # Build release binary for speed
    build = subprocess.run(
        ["cargo", "build", "--bin", BINARY_TARGET, "--release"],
        cwd=PROJECT_ROOT,
        capture_output=True,
        text=True
    )
    if build.returncode != 0:
        print("Build failed:\n", build.stderr)
        return None

    binary = PROJECT_ROOT / "target" / "release" / BINARY_TARGET
    proc = subprocess.run(
        [str(binary), arc_dir],
        capture_output=True,
        text=True
    )
    if proc.returncode != 0:
        print("Binary error:\n", proc.stderr)
        return None
    return json.loads(proc.stdout)

def load_golden_baseline():
    if GOLDEN_BASELINE.exists():
        with open(GOLDEN_BASELINE) as f:
            return json.load(f)
    return {"solved_test": 0, "total_test": 0, "tasks": {}}

def save_golden_baseline(baseline):
    with open(GOLDEN_BASELINE, 'w') as f:
        json.dump(baseline, f, indent=2)

def check_determinism(report):
    """Run the binary 3 times and ensure identical outputs."""
    results = []
    for _ in range(3):
        r = run_binary(str(ARC_DATA_DIR))
        if r is None:
            return False
        results.append(r)
    first = json.dumps(results[0], sort_keys=True)
    for r in results[1:]:
        if json.dumps(r, sort_keys=True) != first:
            return False
    return True

def categorize_tasks(current_report, golden_baseline):
    categories = {"✅ Solved (unchanged)": 0, "🆕 Newly solved": 0,
                  "❌ Previously solved, now broken": 0, "⚠️ Still unsolved": 0}
    golden_tasks = golden_baseline.get("tasks", {})
    for task in current_report.get("tasks", []):
        tid = f"{task['task_file']}_test_{task['test_index']}"
        was_solved = golden_tasks.get(tid, False)
        is_solved = task["solved"]
        if is_solved and was_solved:
            categories["✅ Solved (unchanged)"] += 1
        elif is_solved and not was_solved:
            categories["🆕 Newly solved"] += 1
        elif not is_solved and was_solved:
            categories["❌ Previously solved, now broken"] += 1
        else:
            categories["⚠️ Still unsolved"] += 1
    return categories

def main():
    print("Running Black-Box ARC Regression Suite...")
    report = run_binary(str(ARC_DATA_DIR))
    if report is None:
        sys.exit(1)

    golden = load_golden_baseline()
    cat = categorize_tasks(report, golden)
    det = check_determinism(report)

    print("\n" + "=" * 50)
    print("ARC BLACKBOX REGRESSION")
    print("=" * 50)
    print(f"Training tasks solved: {report['solved_train']}/{report['total_train']}")
    print(f"Test tasks solved: {report['solved_test']}/{report['total_test']}")
    print("\nCategorization:")
    for name, count in cat.items():
        print(f"  {name}: {count}")
    print(f"\nDeterminism: {'PASS' if det else 'FAIL'}")
    regression = cat["❌ Previously solved, now broken"] == 0
    print(f"Regression: {'PASS' if regression else 'FAIL'}")
    overall = det and regression
    print(f"Overall: {'PASS' if overall else 'FAIL'}")

    # Update golden baseline if user approves (manual step)
    if overall:
        new_golden = {
            "version": "0.1.0",
            "date": datetime.now().isoformat(),
            "solved_test": report["solved_test"],
            "total_test": report["total_test"],
            "tasks": {f"{t['task_file']}_test_{t['test_index']}": t["solved"] for t in report["tasks"]}
        }
        print("\nTo update baseline, copy this JSON to golden/baseline.json:")
        print(json.dumps(new_golden, indent=2))
        print("\nRun: python3 tools/update_baseline.py  (if you want to accept)")

    sys.exit(0 if overall else 1)

if __name__ == "__main__":
    main()
