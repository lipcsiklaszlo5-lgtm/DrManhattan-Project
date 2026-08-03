import pathlib

ROOT = pathlib.Path("/workspaces/DrManhattan-Project/manhattan-kernel")
gen_path = ROOT / "src" / "semantic_hypothesis" / "generator.rs"

with open(gen_path, 'r') as f:
    gen = f.read()

# Kicseréljük a hibás step_signature match blokkot
old = """    let target_kind = match &step.target_spec {
        None => "None".to_string(),
        Some(TargetSpec::Constant(_)) => "Constant".to_string(),
        Some(TargetSpec::GridAnchor { corner }) => format!("GridAnchor:{:?}", corner),
        Some(TargetSpec::RelativeToNode { condition, dx_offset: _, dy_offset: _ }) => {
            // Ha van felismert reláció, azt használjuk a szignatúrában
            format!("RelativeToNode:{}", condition.name())
        }
        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string()
    };"""

new = """    let target_kind = match &step.target_spec {
        None => "None".to_string(),
        Some(TargetSpec::Constant(_)) => "Constant".to_string(),
        Some(TargetSpec::GridAnchor { corner }) => format!("GridAnchor:{:?}", corner),
        Some(TargetSpec::RelativeToNode { condition, dx_offset: _, dy_offset: _ }) => {
            format!("RelativeToNode:{}", condition.name())
        },
        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
        Some(TargetSpec::GravitateAnchor { anchor_predicate }) => format!("GravitateAnchor:{}", anchor_predicate.name()),
    };"""

gen = gen.replace(old, new)
gen_path.write_text(gen)
print("step_signature fixed: added GravitateAnchor arm and missing comma")
