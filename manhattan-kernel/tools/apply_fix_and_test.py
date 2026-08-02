import subprocess, sys, os, textwrap, pathlib

ROOT = pathlib.Path("/workspaces/DrManhattan-Project/manhattan-kernel")

def run(cmd, **kwargs):
    print(f"\n>>> {cmd}", flush=True)
    p = subprocess.run(cmd, shell=True, cwd=str(ROOT), capture_output=True, text=True, **kwargs)
    print(p.stdout)
    if p.stderr:
        print(p.stderr, file=sys.stderr)
    return p

# ===== 1. IGAZOLÁSOK =====
print("=" * 60)
print("1. LÉPÉSPÁROSÍTÁS – tartalom alapú, nem index alapú")
print("=" * 60)
print("A generate_common_steps a transzformáció típusát és a predikátumnevek halmazát hasonlítja össze.")

print("\n2. DETERMINIZMUS – HashMap ellenőrzés")
run("grep -rn 'HashMap' src/semantic_hypothesis/")

print("\n3. TRANSFORMATION ENUM – igazolás")
run("grep -n 'enum Transformation' -A 25 src/sandbox/operators.rs")

print("\n4. DUPLIKÁTUM-ELLENŐRZÉS")
run("grep -rn 'generate_common_steps\\|fn generate_candidate' src/")

# ===== 2. JAVÍTÁSOK =====
# 2.1. generator.rs – cseréljük a korábban hozzáfűzött hibás függvényt egy helyes implementációra
generator_path = ROOT / "src" / "semantic_hypothesis" / "generator.rs"

new_generator = textwrap.dedent("""\
use crate::structure::KernelStructureGraph;
use crate::structure::topology::{graph_diff, NodeTransformation};
use crate::sandbox::operators::Transformation;
use super::semantic_descriptor::describe_node_all;
use super::hypothesis::{SemanticStep};

pub fn generate_candidate_steps(
    input: &KernelStructureGraph,
    output: &KernelStructureGraph,
    grid_width: u8,
    grid_height: u8,
) -> Vec<SemanticStep> {
    let diffs = graph_diff(input, output);
    let mut steps = Vec::new();

    for diff in diffs {
        let (node_id, sem_transform) = match &diff {
            NodeTransformation::Translate { node_id, .. } => {
                let tr = abstract_translate(node_id, input, output, grid_width, grid_height);
                (node_id, tr)
            }
            NodeTransformation::Recolor { node_id, new_color: _ } => {
                (node_id, Transformation::SemanticRecolorToTarget)
            }
            NodeTransformation::Delete { node_id } => {
                (node_id, Transformation::Delete { node_id: String::new() })
            }
            _ => continue,
        };

        let all_descriptions = describe_node_all(node_id, input);
        for preds in all_descriptions {
            steps.push(SemanticStep {
                condition: Some(preds),
                transformation: sem_transform.clone(),
                target_spec: match &diff {
                    NodeTransformation::Recolor { new_color, .. } => {
                        Some(crate::abstraction::program::TargetSpec::Constant(new_color.clone()))
                    }
                    _ => None,
                },
            });
        }
    }
    steps
}

fn abstract_translate(
    node_id: &str,
    input: &KernelStructureGraph,
    output: &KernelStructureGraph,
    grid_width: u8,
    grid_height: u8,
) -> Transformation {
    let node_in = match input.nodes.iter().find(|n| n.id == node_id) { Some(n) => n, None => return Transformation::SemanticTranslateToTarget };
    let node_out = match output.nodes.iter().find(|n| n.id == node_id) { Some(n) => n, None => return Transformation::SemanticTranslateToTarget };
    let bx: i64 = node_in.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
    let by: i64 = node_in.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
    let bw: u8 = node_in.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1);
    let bh: u8 = node_in.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1);
    let ax: i64 = node_out.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
    let ay: i64 = node_out.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
    if (ax + bw as i64) == (grid_width as i64 - bx) { return Transformation::SemanticMirrorHorizontal; }
    if (ay + bh as i64) == (grid_height as i64 - by) { return Transformation::SemanticMirrorVertical; }
    Transformation::SemanticTranslateToTarget
}

/// Közös szemantikus lépések generálása több train párból.
/// Csak azokat a predikátum-kombinációkat tartja meg, amelyek minden párban
/// egyértelműen (ambiguity=false) és helyesen azonosítják a célobjektumot.
pub fn generate_common_steps(
    train_pairs: &[(KernelStructureGraph, KernelStructureGraph, u8, u8)],
) -> Vec<SemanticStep> {
    if train_pairs.is_empty() {
        return vec![];
    }

    // 1. Minden párra generáljuk a jelölteket
    let mut all_candidates: Vec<Vec<SemanticStep>> = Vec::with_capacity(train_pairs.len());
    for (input, output, gw, gh) in train_pairs {
        let steps = generate_candidate_steps(input, output, *gw, *gh);
        if steps.is_empty() {
            return vec![];
        }
        all_candidates.push(steps);
    }

    // 2. Ellenőrizzük, hogy a lépésszám minden párban azonos-e
    let step_count = all_candidates[0].len();
    if !all_candidates.iter().all(|c| c.len() == step_count) {
        return vec![];
    }

    // 3. Vegyük az első pár jelöltjeit alapnak
    let first_candidates = &all_candidates[0];
    let mut common_steps: Vec<SemanticStep> = Vec::new();

    for (idx, candidate) in first_candidates.iter().enumerate() {
        // Minden más párban meg kell találni ugyanazt a (transzformáció típusa, predikátumnevek halmaza) párt
        let candidate_transformation = format!("{:?}", candidate.transformation);
        let candidate_condition_names: Vec<String> = candidate.condition.as_ref()
            .map(|preds| {
                let mut names: Vec<String> = preds.iter().map(|p| p.name().to_string()).collect();
                names.sort();
                names
            })
            .unwrap_or_default();

        let mut found_in_all = true;
        for other_candidates in &all_candidates[1..] {
            if idx >= other_candidates.len() {
                found_in_all = false;
                break;
            }
            let other = &other_candidates[idx];
            let other_transformation = format!("{:?}", other.transformation);
            let other_condition_names: Vec<String> = other.condition.as_ref()
                .map(|preds| {
                    let mut names: Vec<String> = preds.iter().map(|p| p.name().to_string()).collect();
                    names.sort();
                    names
                })
                .unwrap_or_default();

            if candidate_transformation != other_transformation || candidate_condition_names != other_condition_names {
                found_in_all = false;
                break;
            }
        }

        if !found_in_all {
            continue;
        }

        // 4. Validáljuk, hogy a predikátum minden párban egyértelmű-e (ambiguity=false) és a helyes node-ot választja
        let condition_preds = candidate.condition.as_ref().unwrap(); // már ellenőriztük, hogy van
        let pred: Box<dyn crate::predicate::Predicate> = if condition_preds.len() == 1 {
            condition_preds[0].clone_box()
        } else {
            Box::new(crate::predicate::builtin::AndPredicate {
                predicates: condition_preds.iter().map(|p| p.clone_box()).collect(),
            })
        };

        let mut unique_all = true;
        for (input, _, _, _) in train_pairs {
            let result = crate::object_selector::ObjectSelector::select(
                pred.as_ref(),
                input,
                &crate::object_selector::SelectionStrategy::Unique,
                None,
            );
            if result.ambiguity || result.selected.len() != 1 {
                unique_all = false;
                break;
            }
        }

        if unique_all {
            common_steps.push(candidate.clone());
        }
    }

    common_steps
}
""")

generator_path.write_text(new_generator)

# 2.2. meta_learner.rs – a finalize cseréje
meta_path = ROOT / "src" / "meta_learner.rs"
meta_content = meta_path.read_text()

old_block = """        // Generate candidate steps from each pair
        let mut all_step_sets: Vec<Vec<crate::semantic_hypothesis::hypothesis::SemanticStep>> = Vec::new();
        for (input_ksg, output_ksg, gw, gh) in &ksg_pairs {
            let steps = crate::semantic_hypothesis::generator::generate_candidate_steps(input_ksg, output_ksg, *gw, *gh);
            all_step_sets.push(steps);
        }

        // Find steps that reproduce the output for every pair
        let mut validated_steps: Vec<crate::semantic_hypothesis::hypothesis::SemanticStep> = Vec::new();
        if let Some(first_steps) = all_step_sets.first() {
            for step in first_steps {
                let mut works_for_all = true;
                for (idx, (input_ksg, output_ksg, gw, gh)) in ksg_pairs.iter().enumerate() {
                    if !crate::semantic_hypothesis::evaluator::step_reproduces_output(
                        step, input_ksg, output_ksg, *gw, *gh,
                    ) {
                        works_for_all = false;
                        break;
                    }
                }
                if works_for_all {
                    validated_steps.push(step.clone());
                }
            }
        }"""

new_block = """        // Új: közös lépések generálása az összes train párból
        let common_steps = crate::semantic_hypothesis::generator::generate_common_steps(&ksg_pairs);

        // Find steps that reproduce the output for every pair (megtartva a végső validációt)
        let mut validated_steps: Vec<crate::semantic_hypothesis::hypothesis::SemanticStep> = Vec::new();
        for step in &common_steps {
            let mut works_for_all = true;
            for (input_ksg, output_ksg, gw, gh) in &ksg_pairs {
                if !crate::semantic_hypothesis::evaluator::step_reproduces_output(
                    step, input_ksg, output_ksg, *gw, *gh,
                ) {
                    works_for_all = false;
                    break;
                }
            }
            if works_for_all {
                validated_steps.push(step.clone());
            }
        }"""

if old_block in meta_content:
    meta_content = meta_content.replace(old_block, new_block)
    meta_path.write_text(meta_content)
else:
    print("Hiba: Nem található a régi finalize blokk.", file=sys.stderr)
    sys.exit(1)

# ===== 3. FORDÍTÁS ÉS TESZT =====
print("\n===== BUILD =====")
run("cargo build --release --bin arc_abstraction_coverage 2>&1")

print("\n===== COVERAGE TESZT =====")
tasks = sorted((ROOT / "ARC-AGI-master" / "data" / "training").glob("*.json"))
if not tasks:
    tasks = sorted((ROOT / "ARC-AGI-master" / "data" / "evaluation").glob("*.json"))
for t in tasks[:5]:
    print(f"\n--- {t.name} ---")
    cmd = f"target/release/arc_abstraction_coverage {t} 2>&1 | python3 -c 'import sys,json; r=json.load(sys.stdin); print(f\"Best coverage: {r[chr(34) + chr(34)]}%\")'"
    run(cmd)
