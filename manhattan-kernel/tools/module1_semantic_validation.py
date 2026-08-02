#!/usr/bin/env python3
"""Module 1 Semantic Validation – proves predicate semantics, composition, and determinism."""

import subprocess, sys, os, time

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def run_test(test_name, filter_str):
    """Futtat egy adott tesztet a megadott szűrővel."""
    cmd = f"cargo test --test semantic_validation -- --nocapture -- {filter_str} 2>&1"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True, cwd=PROJECT_ROOT)
    return result.returncode == 0, result.stdout

def main():
    print("=" * 60)
    print("  MODULE 1 – SEMANTIC VALIDATION")
    print("=" * 60)

    tests = {
        "Largest": "test_largest_predicate_semantics",
        "Smallest": "test_smallest_predicate_semantics",
        "Leftmost": "test_leftmost_predicate_semantics",
        "BorderObject": "test_border_object_semantics",
        "Hole/Inside/Contains": "test_hole_inside_contains_semantics",
        "Nearest/Farthest": "test_nearest_farthest_semantics",
        "Majority/Minority": "test_majority_minority_color_semantics",
        "Composition: AND Largest Red": "test_composition_and_largest_red",
        "Composition: Nested NOT/OR": "test_composition_nested_not_or",
        "Composition: XOR": "test_composition_xor",
        "Composition: IF": "test_composition_if",
    }

    results = {}
    for name, filter_str in tests.items():
        ok, out = run_test(name, filter_str)
        results[name] = ok
        status = "PASS" if ok else "FAIL"
        print(f"  [{status}] {name}")

    print("\n── Determinism ──")
    # Determinizmus teszt külön fájlban van: tests/module1/determinism_test.rs
    cmd = "cargo test --test determinism_test -- --nocapture 2>&1"
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True, cwd=PROJECT_ROOT)
    ok = result.returncode == 0
    results["Determinism (1000 runs)"] = ok
    status = "PASS" if ok else "FAIL"
    print(f"  [{status}] Determinism (1000 runs)")

    print("\n" + "=" * 60)
    passed = sum(results.values())
    total = len(results)
    print(f"  Passed: {passed}/{total}")
    if passed == total:
        print("  MODULE 1 SEMANTICS: VERIFIED")
        print("  READY FOR MODULE 2")
    else:
        print("  MODULE 1 SEMANTICS: ISSUES FOUND")
    print("=" * 60)
    sys.exit(0 if passed == total else 1)

if __name__ == "__main__":
    main()
