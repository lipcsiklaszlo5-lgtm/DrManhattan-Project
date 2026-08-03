import pathlib

ROOT = pathlib.Path("/workspaces/DrManhattan-Project/manhattan-kernel")
gen_path = ROOT / "src" / "semantic_hypothesis" / "generator.rs"

with open(gen_path, 'r') as f:
    content = f.read()

# Fix the broken comma after CopyAttributeFrom
old_broken = """        Some(TargetSpec::CopyAttributeFrom { .. }), => "CopyAttributeFrom".to_string()
    };"""
new_fixed = """        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
        Some(TargetSpec::GravitateAnchor { anchor_predicate }) => format!("GravitateAnchor:{}", anchor_predicate.name()),
    };"""
content = content.replace(old_broken, new_fixed)

# Also fix the unused variable warning by prefixing with underscore
old_rel = "if let Some(rel_name) = relation {"
new_rel = "if let Some(_rel_name) = relation {"
content = content.replace(old_rel, new_rel)

# Fix unused condition variable
old_cond = "let condition = Condition::Predicate(ref_pred.clone_box());"
new_cond = "let _condition = Condition::Predicate(ref_pred.clone_box());"
content = content.replace(old_cond, new_cond)

gen_path.write_text(content)
print("generator.rs: syntax and warnings fixed")
