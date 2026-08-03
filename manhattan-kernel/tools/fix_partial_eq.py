import pathlib, subprocess

ROOT = pathlib.Path("/workspaces/DrManhattan-Project/manhattan-kernel")

def run(cmd):
    print(f">>> {cmd}", flush=True)
    p = subprocess.run(cmd, shell=True, cwd=str(ROOT), capture_output=True, text=True)
    print(p.stdout)
    if p.stderr:
        print(p.stderr)

# 1. Távolítsuk el a PartialEq-et a TargetSpec-ből
prog_path = ROOT / "src" / "abstraction" / "program.rs"
with open(prog_path, 'r') as f:
    prog = f.read()

old_derive = """#[derive(Debug, Clone, PartialEq)]
pub enum TargetSpec {"""
new_derive = """#[derive(Debug, Clone)]
pub enum TargetSpec {"""
prog = prog.replace(old_derive, new_derive)

# 2. Távolítsuk el a PartialEq-et a GridCorner-ből és a Cardinality-ből is, ha szükséges
# (Ezek maradhatnak, mert nincs bennük dyn Predicate)
# Csak a TargetSpec-et kell módosítani

# 3. A step_signature-ben a RelativeToPredicate esetén a predikátum nevét használjuk
gen_path = ROOT / "src" / "semantic_hypothesis" / "generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

# Kicseréljük a step_signature-ben a RelativeToPredicate kezelését
old_sig = """        Some(TargetSpec::RelativeToPredicate { .. }) => "RelativeToPredicate".to_string(),"""
new_sig = """        Some(TargetSpec::RelativeToPredicate { predicate, .. }) => {
            format!("RelativeToPredicate:{}", predicate.name())
        }"""
gen = gen.replace(old_sig, new_sig)

gen_path.write_text(gen)
prog_path.write_text(prog)

# 4. Build & test
run("cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -10")
print("\n===== COVERAGE 017c7c7b =====")
run("target/release/arc_abstraction_coverage ARC-AGI-master/data/training/017c7c7b.json 2>&1")
print("\n===== COMMIT =====")
run("git add -A && git commit -m 'fix: remove PartialEq from TargetSpec due to dyn Predicate' && git push")
