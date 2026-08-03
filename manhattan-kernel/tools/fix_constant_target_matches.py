import pathlib, subprocess

ROOT = pathlib.Path("/workspaces/DrManhattan-Project/manhattan-kernel")

def run(cmd):
    print(f">>> {cmd}", flush=True)
    p = subprocess.run(cmd, shell=True, cwd=str(ROOT), capture_output=True, text=True)
    print(p.stdout)
    if p.stderr:
        print(p.stderr)

gen_path = ROOT / "src" / "semantic_hypothesis" / "generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

old_fn = """fn constant_target_matches(steps: &[&SemanticStep]) -> bool {
    let values: Vec<Option<&String>> = steps.iter().map(|s| match &s.target_spec {
        Some(TargetSpec::Constant(v)) => Some(v),
        _ => None,
    }).collect();
    let has_constant = values.iter().any(|v| v.is_some());
    if !has_constant {
        return true;
    }
    let first = values[0];
    values.iter().all(|v| *v == first)
}"""

new_fn = """fn constant_target_matches(steps: &[&SemanticStep]) -> bool {
    // Extract constant string values, compare as strings
    let strings: Vec<Option<String>> = steps.iter().map(|s| match &s.target_spec {
        Some(TargetSpec::Constant(v)) => Some(v.clone()),
        _ => None,
    }).collect();
    if strings.iter().all(|s| s.is_none()) {
        return true; // no constants, no mismatch possible
    }
    let first = match strings.iter().find(|s| s.is_some()) {
        Some(Some(v)) => v,
        _ => return true,
    };
    strings.iter().all(|s| match s {
        Some(v) => v == first,
        None => true,
    })
}"""

gen = gen.replace(old_fn, new_fn)
gen_path.write_text(gen)

run("cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -10")
print("\n===== COVERAGE 017c7c7b =====")
run("target/release/arc_abstraction_coverage ARC-AGI-master/data/training/017c7c7b.json 2>&1")
print("\n===== COMMIT =====")
run("git add -A && git commit -m 'fix: remove PartialEq from TargetSpec due to dyn Predicate' && git push")
