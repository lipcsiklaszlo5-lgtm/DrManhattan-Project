#!/usr/bin/env python3
"""Module 2 Semantic Validation – Manhattan Kernel

Runs all semantic tests (example-based + property-based) for the
Object Selection Engine. Verifies deterministic, correct behaviour
under all expected ARC scenarios.
"""

import subprocess
import sys
import os
from pathlib import Path
import re

PROJECT_ROOT = Path(__file__).resolve().parent.parent

PASS = "PASS"
FAIL = "FAIL"
results = {}

def run_cargo_test(test_filter: str) -> bool:
    """Run `cargo test --lib -- TEST_FILTER` and return True if all pass."""
    try:
        result = subprocess.run(
            ["cargo", "test", "--lib", "--", test_filter],
            cwd=PROJECT_ROOT,
            capture_output=True,
            text=True
        )
        return result.returncode == 0
    except Exception as e:
        print(f"  ERROR running tests: {e}")
        return False

def run_semantic_suite():
    """Run all semantic test modules and record pass/fail."""
    print("\n[Running Semantic Test Suites]")

    suites = {
        "Candidate Selection": "semantic_selection",
        "Ranking": "semantic_ranking",
        "Scoring": "semantic_scoring",
        "Tie Breaking": "semantic_tiebreak",
        "Multi-Select Strategies": "semantic_multiselect",
        "Determinism (property)": "prop_determinism",
        "Integration": "semantic_integration",
    }

    for name, filter_str in suites.items():
        ok = run_cargo_test(filter_str)
        results[name] = PASS if ok else FAIL
        status = results[name]
        print(f"  {name:.<30} {status}")

def run_property_tests():
    """Property-based tests have their own filter."""
    print("\n[Property-Based Invariants]")
    props = [
        "prop_best_equals_topk_one",
        "prop_topk_subset_of_all",
        "prop_threshold_zero_equals_all",
        "prop_no_duplicates_in_all",
        "prop_determinism_across_runs",
    ]
    all_ok = True
    for prop in props:
        ok = run_cargo_test(prop)
        if not ok:
            all_ok = False
            print(f"  FAILED: {prop}")
    results["Property-Based"] = PASS if all_ok else FAIL
    print(f"  Property-Based Invariants ........ {results['Property-Based']}")

def run_determinism_loop():
    """Run determinism-specific test 1000x (handled inside Rust test)."""
    print("\n[Determinism Loop (1000 iterations)]")
    ok = run_cargo_test("test_1000_iteration_determinism")
    results["Determinism Loop"] = PASS if ok else FAIL
    print(f"  Determinism Loop ............... {results['Determinism Loop']}")

def run_regression():
    """Verify Module 1 tests still pass (regression guard)."""
    print("\n[Regression]")
    ok = run_cargo_test("predicate::tests")
    results["Regression"] = PASS if ok else FAIL
    print(f"  Module 1 Regression ............. {results['Regression']}")

def print_report():
    print("\n" + "=" * 60)
    print("MODULE 2")
    print("SEMANTIC VALIDATION")
    print("=" * 60)
    checks = [
        "Candidate Selection",
        "Ranking",
        "Scoring",
        "Tie Breaking",
        "Multi-Select Strategies",
        "Determinism (property)",
        "Property-Based",
        "Determinism Loop",
        "Integration",
        "Regression",
    ]
    all_pass = True
    for check in checks:
        status = results.get(check, "SKIPPED")
        print(f"{check:.<30} {status}")
        if status != PASS:
            all_pass = False
    print("=" * 60)
    if all_pass:
        print("PASSED")
        print("MODULE 2 SEMANTICS VERIFIED")
        print("READY FOR MODULE 3")
        sys.exit(0)
    else:
        print("FAILED")
        sys.exit(1)

if __name__ == "__main__":
    os.chdir(PROJECT_ROOT)
    print("Running Module 2 Semantic Validation...")
    run_semantic_suite()
    run_property_tests()
    run_determinism_loop()
    run_regression()
    print_report()
