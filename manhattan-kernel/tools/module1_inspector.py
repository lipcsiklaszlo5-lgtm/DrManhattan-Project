#!/usr/bin/env python3
"""
Manhattan Kernel – Module 1 Inspector
Validates Semantic Predicate Engine architecture, API, and completeness.
Reusable for future modules.
"""

import os, sys, re, subprocess, time

class ModuleInspector:
    def __init__(self, project_root=None):
        if project_root is None:
            project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        self.root = project_root
        self.src = os.path.join(self.root, "src")
        self.errors = []
        self.warnings = []
        self.passes = []

    def check_file(self, rel_path, description):
        path = os.path.join(self.src, rel_path)
        if os.path.isfile(path):
            self.passes.append(f"File exists: {description}")
            return True
        else:
            self.errors.append(f"MISSING FILE: {description} ({rel_path})")
            return False

    def check_content(self, rel_path, pattern, description):
        path = os.path.join(self.src, rel_path)
        if not os.path.isfile(path):
            self.errors.append(f"MISSING FILE: {rel_path} (needed for {description})")
            return False
        with open(path, 'r') as f:
            content = f.read()
        if re.search(pattern, content):
            self.passes.append(f"Content found: {description}")
            return True
        else:
            self.errors.append(f"MISSING: {description} in {rel_path}")
            return False

    def check_struct(self, rel_path, struct_name, description):
        return self.check_content(rel_path, rf"pub struct {struct_name}", f"Struct {description}")

    def check_trait(self, rel_path, trait_name, description):
        return self.check_content(rel_path, rf"pub trait {trait_name}", f"Trait {description}")

    def check_enum(self, rel_path, enum_name, description):
        return self.check_content(rel_path, rf"pub enum {enum_name}", f"Enum {description}")

    def run(self):
        print("=" * 60)
        print("  MANHATTAN KERNEL – MODULE 1 INSPECTOR")
        print("=" * 60)
        start = time.time()

        # ── 1. Fájlszerkezet ─────────────────────────────────
        print("\n── 1. File Structure ──")
        self.check_file("predicate/mod.rs", "Predicate module root")
        self.check_file("predicate/builtin.rs", "Builtin predicates")
        self.check_file("predicate/evaluator.rs", "Predicate evaluator")
        self.check_file("predicate/tests.rs", "Predicate tests")
        self.check_file("abstraction/transform.rs", "Condition compatibility wrapper")

        # ── 2. Core abstractions ─────────────────────────────
        print("\n── 2. Core Abstractions ──")
        self.check_trait("predicate/mod.rs", "Predicate", "Predicate trait")
        self.check_enum("predicate/mod.rs", "PredicateResult", "PredicateResult enum")
        self.check_content("predicate/mod.rs", "RankedList", "RankedList variant")
        self.check_content("predicate/mod.rs", "Bool", "Bool variant")
        self.check_content("predicate/mod.rs", "fn evaluate", "evaluate method in trait")
        self.check_content("predicate/mod.rs", "fn name", "name method in trait")
        self.check_content("predicate/mod.rs", "fn clone_box", "clone_box method in trait")
        self.check_content("predicate/evaluator.rs", "pub fn evaluate", "Evaluator entry point")

        # ── 3. Attribute predicates ──────────────────────────
        print("\n── 3. Attribute Predicates ──")
        attrs = {
            "ColorPredicate": "Color",
            "AreaPredicate": "Area",
            "WidthPredicate": "Width",
            "HeightPredicate": "Height",
            "RolePredicate": "Role",
            "ShapePredicate": "Shape",
            "PixelCountPredicate": "Pixel count",
            "AspectRatioPredicate": "Aspect ratio",
            "BoundingBoxPredicate": "Bounding box",
        }
        for struct_name, desc in attrs.items():
            self.check_struct("predicate/builtin.rs", struct_name, desc)

        # ── 4. Relative predicates ───────────────────────────
        print("\n── 4. Relative Predicates ──")
        relatives = {
            "LeftOfPredicate": "LeftOf",
            "RightOfPredicate": "RightOf",
            "AbovePredicate": "Above",
            "BelowPredicate": "Below",
            "AdjacentPredicate": "Adjacent",
            "ConnectedPredicate": "Connected",
            "InsidePredicate": "Inside",
            "ContainsPredicate": "Contains",
            "NearestPredicate": "Nearest",
            "FarthestPredicate": "Farthest",
            "IntersectsPredicate": "Intersects",
        }
        for struct_name, desc in relatives.items():
            self.check_struct("predicate/builtin.rs", struct_name, desc)

        # ── 5. Global predicates ─────────────────────────────
        print("\n── 5. Global Predicates ──")
        globals_ = {
            "LargestPredicate": "Largest",
            "SmallestPredicate": "Smallest",
            "LeftmostPredicate": "Leftmost",
            "RightmostPredicate": "Rightmost",
            "TopmostPredicate": "Topmost",
            "BottommostPredicate": "Bottommost",
            "OnlyObjectPredicate": "OnlyObject",
            "UniqueColorPredicate": "UniqueColor",
            "MajorityColorPredicate": "MajorityColor",
            "MinorityColorPredicate": "MinorityColor",
            "CenterObjectPredicate": "CenterObject",
            "CornerObjectPredicate": "CornerObject",
            "BorderObjectPredicate": "BorderObject",
        }
        for struct_name, desc in globals_.items():
            self.check_struct("predicate/builtin.rs", struct_name, desc)

        # ── 6. Shape predicates ──────────────────────────────
        print("\n── 6. Shape Predicates ──")
        shapes = {
            "SymmetryPredicate": "Symmetry",
            "MirrorSymmetricPredicate": "MirrorSymmetric",
            "RotationalSymmetryPredicate": "RotationalSymmetry",
            "RectanglePredicate": "Rectangle",
            "LinePredicate": "Line",
            "PointPredicate": "Point",
            "CrossPredicate": "Cross",
            "HolePredicate": "Hole",
            "ConvexPredicate": "Convex",
            "ConcavePredicate": "Concave",
            "FilledPredicate": "Filled",
            "HollowPredicate": "Hollow",
        }
        for struct_name, desc in shapes.items():
            self.check_struct("predicate/builtin.rs", struct_name, desc)

        # ── 7. Quantitative predicates ───────────────────────
        print("\n── 7. Quantitative Predicates ──")
        quants = {
            "EqualAreaPredicate": "EqualArea",
            "EqualShapePredicate": "EqualShape",
            "EqualColorPredicate": "EqualColor",
            "EqualWidthPredicate": "EqualWidth",
            "EqualHeightPredicate": "EqualHeight",
            "EqualNeighbourCountPredicate": "EqualNeighbourCount",
            "ObjectCountPredicate": "ObjectCount",
            "NeighbourCountPredicate": "NeighbourCount",
        }
        for struct_name, desc in quants.items():
            self.check_struct("predicate/builtin.rs", struct_name, desc)

        # ── 8. Logical predicates ────────────────────────────
        print("\n── 8. Logical Predicates ──")
        logicals = {
            "AndPredicate": "AND",
            "OrPredicate": "OR",
            "NotPredicate": "NOT",
            "XorPredicate": "XOR",
            "IfPredicate": "IF",
        }
        for struct_name, desc in logicals.items():
            self.check_struct("predicate/builtin.rs", struct_name, desc)

        # ── 9. PredicateResult methods ───────────────────────
        print("\n── 9. PredicateResult API ──")
        self.check_content("predicate/mod.rs", "fn as_bool", "as_bool()")
        self.check_content("predicate/mod.rs", "fn as_ranked_list", "as_ranked_list()")
        self.check_content("predicate/mod.rs", "fn is_true", "is_true()")
        self.check_content("predicate/mod.rs", "fn len", "len()")

        # ── 10. Compatibility ────────────────────────────────
        print("\n── 10. Backward Compatibility ──")
        self.check_content("abstraction/transform.rs", "impl Predicate for Condition",
                           "Condition implements Predicate")
        self.check_content("abstraction/transform.rs", "Condition::AlwaysTrue",
                           "Condition::AlwaysTrue exists")
        self.check_content("abstraction/transform.rs", "Condition::NodeHasAttribute",
                           "Condition::NodeHasAttribute exists")
        self.check_content("abstraction/transform.rs", "Condition::ColorEquals",
                           "Condition::ColorEquals exists")
        self.check_content("abstraction/transform.rs", "Condition::PositionAbove",
                           "Condition::PositionAbove exists")
        self.check_content("abstraction/transform.rs", "Condition::PositionLeftOf",
                           "Condition::PositionLeftOf exists")

        # ── 11. Integration ──────────────────────────────────
        print("\n── 11. Integration Points ──")
        self.check_content("abstraction/program.rs", "use crate::predicate",
                           "program.rs imports predicate")
        self.check_content("abstraction/program.rs", "GeneralizedProgram",
                           "GeneralizedProgram exists")
        self.check_content("abstraction/program.rs", "AbstractStep",
                           "AbstractStep exists")
        self.check_content("abstraction/program.rs", "condition: Option<Box<dyn Predicate>>",
                           "AbstractStep uses Predicate")
        self.check_content("abstraction/hypothesis.rs", "HypothesisManager",
                           "HypothesisManager exists")
        self.check_content("abstraction/goal_decomposer.rs", "GoalDecomposer",
                           "GoalDecomposer exists")
        self.check_content("abstraction/representation.rs", "RepresentationFactory",
                           "RepresentationFactory exists")
        self.check_content("concept/mod.rs", "ConceptRegistry",
                           "ConceptRegistry exists")
        self.check_content("concept/mod.rs", "fn to_predicates",
                           "ConceptRegistry::to_predicates")

        # ── 12. Build check ──────────────────────────────────
        print("\n── 12. Build Check ──")
        result = subprocess.run(
            ["cargo", "build", "--lib"],
            cwd=self.root,
            capture_output=True,
            text=True
        )
        if result.returncode == 0:
            self.passes.append("Build successful")
        else:
            self.errors.append(f"Build FAILED:\n{result.stderr[-500:]}")

        # ── 13. Test check ───────────────────────────────────
        print("\n── 13. Unit Tests ──")
        result = subprocess.run(
            ["cargo", "test", "--lib"],
            cwd=self.root,
            capture_output=True,
            text=True
        )
        if result.returncode == 0:
            # Kivonjuk a teszt eredményt
            match = re.search(r"test result: ok\. (\d+) passed", result.stdout)
            if match:
                count = match.group(1)
                self.passes.append(f"All {count} tests passed")
            else:
                self.passes.append("All tests passed")
        else:
            self.errors.append(f"Tests FAILED:\n{result.stderr[-500:]}")

        # ── 14. ObjectSelector integration ──────────────────
        print("\n── 14. ObjectSelector ──")
        self.check_file("object_selector.rs", "ObjectSelector module")
        self.check_content("object_selector.rs", "pub fn select", "select method")
        self.check_content("object_selector.rs", "pub fn select_all", "select_all method")

        # ── Summary ──────────────────────────────────────────
        elapsed = time.time() - start
        print("\n" + "=" * 60)
        print(f"  INSPECTION COMPLETE ({elapsed:.1f}s)")
        print("=" * 60)
        total = len(self.passes) + len(self.errors)
        print(f"  Passed: {len(self.passes)}/{total}")
        print(f"  Failed: {len(self.errors)}/{total}")
        if self.warnings:
            print(f"  Warnings: {len(self.warnings)}")
        print("=" * 60)

        if self.errors:
            print("\n  ERRORS:")
            for e in self.errors:
                print(f"    ✗ {e}")
        if self.warnings:
            print("\n  WARNINGS:")
            for w in self.warnings:
                print(f"    ⚠ {w}")
        print()
        return len(self.errors) == 0

if __name__ == "__main__":
    inspector = ModuleInspector()
    success = inspector.run()
    sys.exit(0 if success else 1)
