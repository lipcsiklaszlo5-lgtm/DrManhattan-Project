import pathlib, subprocess

ROOT = pathlib.Path("/workspaces/DrManhattan-Project/manhattan-kernel")

def run(cmd):
    print(f">>> {cmd}", flush=True)
    p = subprocess.run(cmd, shell=True, cwd=str(ROOT), capture_output=True, text=True)
    print(p.stdout)
    if p.stderr:
        print(p.stderr)

# 1. Új TargetSpec variáns hozzáadása a program.rs-hez
prog_path = ROOT / "src" / "abstraction" / "program.rs"
with open(prog_path, 'r') as f:
    prog = f.read()

# Új variáns beszúrása a TargetSpec enum-ba (a RelativeToNode után)
old_enum = """    RelativeToNode { condition: Box<Condition>, dx_offset: i64, dy_offset: i64 },"""
new_enum = """    RelativeToNode { condition: Box<Condition>, dx_offset: i64, dy_offset: i64 },
    /// Új: predikátum-alapú relatív pozícionálás (SHE-kompatibilis)
    RelativeToPredicate { predicate: Box<dyn Predicate>, dx_offset: i64, dy_offset: i64 },"""
prog = prog.replace(old_enum, new_enum)

# A resolve_target_spec-ben hozzáadjuk az új variáns kezelését
old_resolve = """            TargetSpec::CopyAttributeFrom { condition, attribute } => {
                let refs = Self::matching_nodes(graph, condition.as_ref());
                if let Some(ref_node) = refs.first() {
                    let val = ref_node.attributes.get(attribute).cloned();
                    Some((0, 0, val))
                } else { None }
            }"""
new_resolve = """            TargetSpec::CopyAttributeFrom { condition, attribute } => {
                let refs = Self::matching_nodes(graph, condition.as_ref());
                if let Some(ref_node) = refs.first() {
                    let val = ref_node.attributes.get(attribute).cloned();
                    Some((0, 0, val))
                } else { None }
            }
            TargetSpec::RelativeToPredicate { predicate, dx_offset, dy_offset } => {
                let refs = Self::matching_nodes(graph, predicate.as_ref());
                if let Some(ref_node) = refs.first() {
                    let rx: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let ry: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                    Some((rx + dx_offset, ry + dy_offset, None))
                } else { None }
            }"""
prog = prog.replace(old_resolve, new_resolve)

# A step_signature-ben (generator.rs) is hivatkozunk a TargetSpec variánsokra, frissíteni kell ott is
# Ezt a generator.rs-ben fogjuk megtenni.

prog_path.write_text(prog)

# 2. Generator.rs módosítása: relatív célpont generálása, ha nincs sarok
gen_path = ROOT / "src" / "semantic_hypothesis" / "generator.rs"
with open(gen_path, 'r') as f:
    gen = f.read()

# Frissítjük a step_signature függvényt, hogy kezelje az új variánst
old_sig = """        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),"""
new_sig = """        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
        Some(TargetSpec::RelativeToPredicate { .. }) => "RelativeToPredicate".to_string(),"""
gen = gen.replace(old_sig, new_sig)

# A generate_candidate_steps Translate ágában a grid_anchor_for_node után beszúrjuk a relatív célpont generálást
# Megkeressük azt a részt, ahol a None ágon continue van
old_translate = """                        match grid_anchor_for_node(node_out, grid_width, grid_height) {
                            Some(spec) => (Transformation::SemanticTranslateToTarget, Some(spec)),
                            None => continue,
                        }"""
new_translate = """                        match grid_anchor_for_node(node_out, grid_width, grid_height) {
                            Some(spec) => (Transformation::SemanticTranslateToTarget, Some(spec)),
                            None => {
                                // Próbáljunk relatív célpontot generálni egy referencia objektumhoz képest
                                if let Some(ref_node) = input.nodes.iter()
                                    .filter(|n| n.id != *node_id)
                                    .max_by_key(|n| n.attributes.get("area").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0))
                                {
                                    if let Some(ref_preds) = describe_node_all(&ref_node.id, input).into_iter().next() {
                                        let ref_x: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                                        let ref_y: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                                        let ax: i64 = node_out.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                                        let ay: i64 = node_out.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                                        let rel_dx = ax - ref_x;
                                        let rel_dy = ay - ref_y;
                                        let ref_pred: Box<dyn Predicate> = if ref_preds.len() == 1 {
                                            ref_preds[0].clone_box()
                                        } else {
                                            Box::new(crate::predicate::builtin::AndPredicate {
                                                predicates: ref_preds.iter().map(|p| p.clone_box()).collect(),
                                            })
                                        };
                                        (Transformation::SemanticTranslateToTarget, Some(TargetSpec::RelativeToPredicate {
                                            predicate: ref_pred,
                                            dx_offset: rel_dx,
                                            dy_offset: rel_dy,
                                        }))
                                    } else {
                                        continue
                                    }
                                } else {
                                    continue
                                }
                            }
                        }"""
gen = gen.replace(old_translate, new_translate)

gen_path.write_text(gen)

# 3. Build & test
run("cargo build --release --bin arc_abstraction_coverage 2>&1 | tail -10")
print("\n===== COVERAGE 017c7c7b =====")
run("target/release/arc_abstraction_coverage ARC-AGI-master/data/training/017c7c7b.json 2>&1")
print("\n===== COMMIT =====")
run("git add -A && git commit -m 'feat: add RelativeToPredicate target spec for semantic translate' && git push")
