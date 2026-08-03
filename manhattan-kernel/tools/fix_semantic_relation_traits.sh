#!/bin/bash
set -e
cd /workspaces/DrManhattan-Project/manhattan-kernel

# 1. SpatialRelation duplikátum eltávolítása, kézi trait-ek a TargetSpec-re
python3 << 'PYEOF'
prog_path = "src/abstraction/program.rs"
with open(prog_path, 'r') as f:
    prog = f.read()

# Eltávolítjuk a SpatialRelation duplikátumát (ha kétszer szerepel)
# Keressük az első előfordulást, és töröljük a másodikat
first_idx = prog.find("pub enum SpatialRelation {")
if first_idx != -1:
    second_idx = prog.find("pub enum SpatialRelation {", first_idx + 1)
    if second_idx != -1:
        # Töröljük a második előfordulást (a hozzá tartozó derive-okkal együtt)
        end_idx = prog.find("}\n", second_idx)
        if end_idx != -1:
            prog = prog[:second_idx] + prog[end_idx+2:]

# TargetSpec derive-ok cseréje: Debug + Clone marad, PartialEq eltávolítva
prog = prog.replace("#[derive(Debug, Clone)]\npub enum TargetSpec {",
                      "pub enum TargetSpec {")

# Kézi Debug a TargetSpec-re
debug_impl = """
impl std::fmt::Debug for TargetSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetSpec::Constant(v) => write!(f, "Constant({})", v),
            TargetSpec::RelativeToNode { condition, dx_offset, dy_offset } =>
                write!(f, "RelativeToNode({}, {}, {})", condition.name(), dx_offset, dy_offset),
            TargetSpec::GridAnchor { corner } => write!(f, "GridAnchor({:?})", corner),
            TargetSpec::CopyAttributeFrom { condition, attribute } =>
                write!(f, "CopyAttributeFrom({}, {})", condition.name(), attribute),
            TargetSpec::SemanticRelation { relation, anchor_predicate } =>
                write!(f, "SemanticRelation({:?}, {})", relation, anchor_predicate.name()),
        }
    }
}
"""

# Kézi Clone a TargetSpec-re
clone_impl = """
impl Clone for TargetSpec {
    fn clone(&self) -> Self {
        match self {
            TargetSpec::Constant(v) => TargetSpec::Constant(v.clone()),
            TargetSpec::RelativeToNode { condition, dx_offset, dy_offset } =>
                TargetSpec::RelativeToNode { condition: condition.clone(), dx_offset: *dx_offset, dy_offset: *dy_offset },
            TargetSpec::GridAnchor { corner } => TargetSpec::GridAnchor { corner: corner.clone() },
            TargetSpec::CopyAttributeFrom { condition, attribute } =>
                TargetSpec::CopyAttributeFrom { condition: condition.clone(), attribute: attribute.clone() },
            TargetSpec::SemanticRelation { relation, anchor_predicate } =>
                TargetSpec::SemanticRelation { relation: relation.clone(), anchor_predicate: anchor_predicate.clone_box() },
        }
    }
}
"""

# Beszúrjuk a kézi implementációkat a TargetSpec után
prog = prog.replace("\n#[derive(Debug, Clone, PartialEq)]\npub enum GridCorner",
                      debug_impl + "\n" + clone_impl + "\n\n#[derive(Debug, Clone, PartialEq)]\npub enum GridCorner")

with open(prog_path, 'w') as f:
    f.write(prog)
print("program.rs: duplicates removed, manual Debug/Clone for TargetSpec added")
PYEOF

# 2. generator.rs: step_signature match kiegészítése
python3 << 'PYEOF'
gen_path = "src/semantic_hypothesis/generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

old_sig = """        Some(TargetSpec::SemanticRelation { relation, .. }) => format!("SemanticRelation:{:?}", relation),
        _ => "Unknown".to_string(),
    }"""
new_sig = """        Some(TargetSpec::SemanticRelation { relation, .. }) => format!("SemanticRelation:{:?}", relation),
    };
    (transformation_shape, cond_names, target_kind)
}"""
# Csak az utolsó sort cseréljük, a többit meghagyjuk
gen = gen.replace(old_sig, new_sig)

with open(gen_path, 'w') as f:
    f.write(gen)
print("generator.rs: step_signature fixed")
PYEOF

# 3. Build
echo "===== BUILD ====="
cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -15
echo "===== COMMIT ====="
git add -A && git commit -m "fix: remove duplicate SpatialRelation, manual Debug/Clone for TargetSpec" && git push
