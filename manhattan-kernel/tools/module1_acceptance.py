#!/usr/bin/env python3
"""
Manhattan Kernel – Module 1 Acceptance Test
Runs the inspector and all integration tests.
"""

import subprocess, sys, os, time

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def run_step(cmd, description):
    print(f"\n--- {description} ---")
    start = time.time()
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True, cwd=PROJECT_ROOT)
    elapsed = time.time() - start
    if result.returncode != 0:
        print(f"  FAIL ({elapsed:.1f}s)")
        if result.stdout:
            print("  STDOUT:", result.stdout[-500:])
        if result.stderr:
            print("  STDERR:", result.stderr[-500:])
        return False
    print(f"  PASS ({elapsed:.1f}s)")
    return True

def main():
    print("=" * 60)
    print("  MANHATTAN KERNEL")
    print("  MODULE 1 ACCEPTANCE TEST")
    print("=" * 60)

    results = {}

    # 1. Inspector
    results["Inspector"] = run_step(
        f"{sys.executable} tools/module1_inspector.py",
        "Module Inspector"
    )

    # 2. Full test suite
    results["Full Test Suite"] = run_step(
        "cargo test 2>&1",
        "Full Test Suite (cargo test)"
    )

    # Summary
    print("\n" + "=" * 60)
    all_pass = all(results.values())
    for name, passed in results.items():
        status = "PASS" if passed else "FAIL"
        print(f"  {name:<30} {status}")
    print("=" * 60)
    if all_pass:
        print("  MODULE 1: READY")
        print("  NEXT: MODULE 2 (Object Selection Engine)")
    else:
        print("  MODULE 1: ISSUES FOUND")
    print("=" * 60)
    sys.exit(0 if all_pass else 1)

if __name__ == "__main__":
    main()
