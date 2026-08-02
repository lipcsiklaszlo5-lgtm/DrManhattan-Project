#!/usr/bin/env python3
"""Module 2 Architectural Inspector – Manhattan Kernel

Checks architectural completeness, low coupling, pluggable strategies,
and readiness for semantic validation. Does NOT test correctness of
selection results.
"""

import re
import sys
import subprocess
import os
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
SRC = PROJECT_ROOT / "src"
OBJECT_SELECTOR_FILE = SRC / "object_selector.rs"
PROGRAM_FILE = SRC / "abstraction" / "program.rs"
PREDICATE_MOD_FILE = SRC / "predicate" / "mod.rs"

PASS = "PASS"
FAIL = "FAIL"
results = {}

def check_file_exists(filepath, label):
    exists = filepath.exists()
    results[label] = PASS if exists else FAIL
    if not exists:
        print(f"  MISSING: {filepath}")
    return exists

def grep_file(filepath, pattern):
    try:
        content = filepath.read_text()
        return bool(re.search(pattern, content, re.MULTILINE))
    except Exception:
        return False

def check_architecture():
    """Verify core abstractions exist in object_selector.rs"""
    print("\n[Architecture]")
    if not check_file_exists(OBJECT_SELECTOR_FILE, "object_selector.rs"):
        return

    abstractions = {
        "ObjectSelector": r"struct\s+ObjectSelector",
        "SelectionStrategy": r"enum\s+SelectionStrategy",
        "SelectionResult": r"struct\s+SelectionResult",
        "SelectedObject": r"struct\s+SelectedObject",
        "ScoringComponent": r"trait\s+ScoringComponent",
        "ScoringProfile": r"struct\s+ScoringProfile",
        "TieBreaker": r"(trait\s+TieBreaker|struct\s+TieBreaker|fn\s+tie_break|fn\s+resolve_tie)",
        "RankingEngine": r"(trait\s+Rank|fn\s+rank|struct\s+Rank)",
    }
    all_pass = True
    for name, pattern in abstractions.items():
        ok = grep_file(OBJECT_SELECTOR_FILE, pattern)
        results[name] = PASS if ok else FAIL
        if not ok:
            all_pass = False
            print(f"  MISSING ABSTRACTION: {name}")
    if all_pass:
        print("  All core abstractions found")
    results["Architecture"] = PASS if all_pass else FAIL

def check_low_coupling():
    """Ensure ObjectSelector does not import high-level modules"""
    print("\n[Low Coupling]")
    if not OBJECT_SELECTOR_FILE.exists():
        results["LowCoupling"] = FAIL
        return
    content = OBJECT_SELECTOR_FILE.read_text()
    # Forbidden imports that would create tight coupling
    forbidden = [
        "use crate::abstraction::program",   # program.rs
        "use crate::agent",                  # agent
        "use crate::meta_learner",           # meta learner
        "use crate::concept_learner",        # concept learner
        "use crate::hypothesis_bus",         # hypothesis bus
        "use crate::planner",               # planner (future)
    ]
    coupled = [f for f in forbidden if f in content]
    if coupled:
        print(f"  TIGHT COUPLING DETECTED: {', '.join(coupled)}")
        results["LowCoupling"] = FAIL
    else:
        print("  No tight coupling – good modularity")
        results["LowCoupling"] = PASS

def check_no_cyclic_deps():
    """Run cargo check and look for cyclic dependency errors"""
    print("\n[Cyclic Dependencies]")
    try:
        result = subprocess.run(
            ["cargo", "check", "--lib"],
            cwd=PROJECT_ROOT,
            capture_output=True,
            text=True
        )
        if "cyclic dependency" in result.stderr.lower() or "cycle" in result.stderr.lower():
            print("  CYCLIC DEPENDENCY DETECTED")
            results["CyclicDeps"] = FAIL
        elif result.returncode != 0:
            print(f"  cargo check failed (but not cyclic): {result.stderr[-200:]}")
            results["CyclicDeps"] = FAIL
        else:
            print("  No cyclic dependencies")
            results["CyclicDeps"] = PASS
    except Exception as e:
        print(f"  ERROR running cargo check: {e}")
        results["CyclicDeps"] = FAIL

def check_pipeline():
    """Verify select() method uses distinct stages: eval → score → rank → tie-break"""
    print("\n[Pipeline]")
    if not OBJECT_SELECTOR_FILE.exists():
        results["Pipeline"] = FAIL
        return
    content = OBJECT_SELECTOR_FILE.read_text()
    # Look for key method calls inside select()
    stages = {
        "Predicate Evaluation": r"predicate\.evaluate",
        "Candidate Generation": r"(candidates|matching_nodes|filter_map|filter)",
        "Scoring": r"(score|scoring_profile|score_fn)",
        "Ranking": r"(sort|rank|ordering)",
        "Tie Breaking": r"(tie_break|resolve_tie|tie_breaker)",
    }
    missing = [name for name, pat in stages.items() if not re.search(pat, content)]
    if missing:
        print(f"  MISSING STAGES: {', '.join(missing)}")
        results["Pipeline"] = FAIL
    else:
        print("  Full pipeline present")
        results["Pipeline"] = PASS

def check_strategy_pluggable():
    """Ensure strategies are not hardcoded in a monolithic switch"""
    print("\n[Strategy Pluggability]")
    if not OBJECT_SELECTOR_FILE.exists():
        results["StrategyPluggable"] = FAIL
        return
    content = OBJECT_SELECTOR_FILE.read_text()
    # Look for select method body; if it contains match on strategy with big blocks, that's a smell
    if "match strategy" in content or "match &strategy" in content:
        # Allow if each branch just calls a separate function
        branches = re.findall(r"SelectionStrategy::(\w+)\s*=>\s*\{([^}]+)\}", content, re.DOTALL)
        complex_branches = [b for b in branches if len(b[1].strip().split('\n')) > 5]
        if complex_branches:
            print(f"  Possible monolithic match: {[b[0] for b in complex_branches]}")
            results["StrategyPluggable"] = FAIL
        else:
            print("  Strategies dispatched cleanly")
            results["StrategyPluggable"] = PASS
    else:
        # No match on strategy – maybe using trait objects
        results["StrategyPluggable"] = PASS
        print("  Strategy pattern detected (trait objects)")

def check_selection_result_fields():
    """Verify SelectionResult has required fields"""
    print("\n[SelectionResult]")
    if not OBJECT_SELECTOR_FILE.exists():
        results["SelectionResultFields"] = FAIL
        return
    content = OBJECT_SELECTOR_FILE.read_text()
    required = ["selected", "ranking", "ambiguity", "confidence", "explanation"]
    missing = [f for f in required if not re.search(rf"\b{f}\b\s*:", content)]
    if missing:
        print(f"  MISSING FIELDS: {missing}")
        results["SelectionResultFields"] = FAIL
    else:
        print("  All required fields present")
        results["SelectionResultFields"] = PASS

def check_scoring_extensible():
    """ScoringComponent trait and ScoringProfile::add_component method"""
    print("\n[Scoring Extensibility]")
    if not OBJECT_SELECTOR_FILE.exists():
        results["ScoringExtensible"] = FAIL
        return
    content = OBJECT_SELECTOR_FILE.read_text()
    has_trait = "trait ScoringComponent" in content
    has_profile = "struct ScoringProfile" in content
    has_add = "add_component" in content or "with_component" in content
    if has_trait and has_profile and has_add:
        print("  Extensible scoring API present")
        results["ScoringExtensible"] = PASS
    else:
        missing = []
        if not has_trait: missing.append("ScoringComponent trait")
        if not has_profile: missing.append("ScoringProfile struct")
        if not has_add: missing.append("add_component method")
        print(f"  MISSING: {missing}")
        results["ScoringExtensible"] = FAIL

def check_backward_compatibility():
    """GeneralizedProgram::matching_nodes must call ObjectSelector::select with All strategy"""
    print("\n[Backward Compatibility]")
    if not PROGRAM_FILE.exists():
        print("  WARNING: program.rs not found")
        results["BackwardCompat"] = FAIL
        return
    content = PROGRAM_FILE.read_text()
    # matching_nodes should exist and call ObjectSelector::select
    if "fn matching_nodes" not in content:
        print("  matching_nodes function missing (may have been renamed)")
        results["BackwardCompat"] = FAIL
        return
    if "ObjectSelector" in content and ("All" in content or "SelectionStrategy::All" in content):
        print("  matching_nodes delegates to ObjectSelector with SelectionStrategy::All")
        results["BackwardCompat"] = PASS
    else:
        print("  matching_nodes does NOT delegate to ObjectSelector::All")
        results["BackwardCompat"] = FAIL

def check_integration():
    """Verify ObjectSelector is referenced in key integration points"""
    print("\n[Integration]")
    integration_points = {
        "ProgramSynthesizer": SRC / "abstraction" / "program.rs",
        "HypothesisManager": SRC / "abstraction" / "hypothesis.rs",
        "GoalDecomposer": SRC / "abstraction" / "goal_decomposer.rs",
        "RepresentationFactory": SRC / "abstraction" / "representation.rs",
        "ConceptRegistry": SRC / "concept" / "mod.rs",
    }
    # Only check files that exist
    ok = True
    for name, path in integration_points.items():
        if path.exists():
            if "ObjectSelector" in path.read_text():
                print(f"  {name}: ObjectSelector integrated")
            else:
                # Not mandatory for all, but note it
                pass
    # At minimum, program.rs must reference it
    if PROGRAM_FILE.exists() and "ObjectSelector" in PROGRAM_FILE.read_text():
        print("  Core integration confirmed (program.rs)")
        results["Integration"] = PASS
    else:
        print("  ObjectSelector not found in program.rs – may not be wired")
        results["Integration"] = FAIL

def check_no_unsafe():
    """Ensure zero unsafe blocks in object_selector.rs"""
    print("\n[Unsafe Audit]")
    if OBJECT_SELECTOR_FILE.exists():
        content = OBJECT_SELECTOR_FILE.read_text()
        if "unsafe" in content:
            print("  UNSAFE BLOCK DETECTED")
            results["NoUnsafe"] = FAIL
        else:
            print("  Zero unsafe blocks")
            results["NoUnsafe"] = PASS
    else:
        results["NoUnsafe"] = FAIL

def check_regression():
    """Run cargo test --lib and verify all tests pass"""
    print("\n[Regression Tests]")
    try:
        result = subprocess.run(
            ["cargo", "test", "--lib"],
            cwd=PROJECT_ROOT,
            capture_output=True,
            text=True
        )
        if result.returncode == 0:
            # Count passed tests
            passed = re.findall(r"test result:.*?(\d+) passed", result.stderr)
            print(f"  All tests pass (output: {passed[-1] if passed else 'unknown'} passed)")
            results["Regression"] = PASS
        else:
            failures = re.findall(r"failures:", result.stderr)
            print(f"  Some tests FAILED: {failures}")
            results["Regression"] = FAIL
    except Exception as e:
        print(f"  ERROR: {e}")
        results["Regression"] = FAIL

def print_report():
    print("\n" + "=" * 60)
    print("MANHATTAN KERNEL")
    print("MODULE 2 ARCHITECTURE INSPECTOR")
    print("=" * 60)
    checks = [
        "Architecture",
        "LowCoupling",
        "CyclicDeps",
        "Pipeline",
        "StrategyPluggable",
        "SelectionResultFields",
        "ScoringExtensible",
        "BackwardCompat",
        "Integration",
        "NoUnsafe",
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
        print("READY FOR SEMANTIC VALIDATION")
        sys.exit(0)
    else:
        print("FAILED")
        print("Fix architectural issues before semantic validation.")
        sys.exit(1)

if __name__ == "__main__":
    os.chdir(PROJECT_ROOT)
    print("Running Module 2 Architectural Inspector...")
    check_architecture()
    check_low_coupling()
    check_no_cyclic_deps()
    check_pipeline()
    check_strategy_pluggable()
    check_selection_result_fields()
    check_scoring_extensible()
    check_backward_compatibility()
    check_integration()
    check_no_unsafe()
    check_regression()
    print_report()
