use crate::structure::KernelStructureGraph;
use crate::structure::topology::{graph_diff, NodeTransformation};
use crate::sandbox::operators::Transformation;
use super::semantic_descriptor::describe_node_all;
use super::hypothesis::{SemanticStep};
use crate::object_selector::ObjectSelector;
use crate::predicate::Predicate;
use crate::abstraction::program::TargetSpec;
use std::collections::HashMap as StdHashMap;

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
                        Some(TargetSpec::Constant(new_color.clone()))
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

/// Egy jelolt-lepes "alairasa": a transzformacio ALAKJA (nem a konkret erteke,
/// pl. SemanticRecolorToTarget mindig ugyanugy nez ki fuggetlenul a celszintol)
/// plusz a feltetel-predikatumok NEVEINEK rendezett halmaza. Ket lepes akkor
/// "ugyanaz", ha ez az alairas megegyezik -- FUGGETLENUL attol, hanyadik
/// helyen allnak a jelolt-listaban (a describe_node_all parononkent eltero
/// szamu alternativat adhat vissza, ezert a POZICIO szerinti egyeztetes
/// megbizhatatlan volt).
fn step_signature(step: &SemanticStep) -> (String, Vec<String>) {
    let transformation_shape = format!("{:?}", step.transformation);
    let mut cond_names: Vec<String> = step.condition.as_ref()
        .map(|preds| preds.iter().map(|p| p.name().to_string()).collect())
        .unwrap_or_default();
    cond_names.sort();
    (transformation_shape, cond_names)
}

/// Ha a lepes target_spec-je Constant(...), az ertekenek meg kell egyeznie
/// MINDEN parban -- kulonben a lepes nem egy valodi altalanos szabaly, hanem
/// csak veletlen egybeeses volt az adott parban.
fn constant_target_matches(steps: &[&SemanticStep]) -> bool {
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
}

/// Közös szemantikus lépések generálása több train párból.
/// Halmaz-alapu (nem pozicio-alapu) egyeztetes: minden parra osszegyujtjuk a
/// jelolt lepesek "alairasait", majd csak azokat tartjuk meg, amik MINDEN
/// parban elofordulnak, ES ha van konstans celertek, az is egyezik mindenhol,
/// ES a feltetel minden parban egyertelmuen (ambiguity=false) egy node-ot
/// valaszt ki.
pub fn generate_common_steps(
    train_pairs: &[(KernelStructureGraph, KernelStructureGraph, u8, u8)],
) -> Vec<SemanticStep> {
    let diag = std::env::var("MK_DIAG").is_ok();
    if train_pairs.is_empty() {
        return vec![];
    }

    let mut all_candidates: Vec<Vec<SemanticStep>> = Vec::with_capacity(train_pairs.len());
    for (idx, (input, output, gw, gh)) in train_pairs.iter().enumerate() {
        let steps = generate_candidate_steps(input, output, *gw, *gh);
        if diag {
            eprintln!("[DIAG] pair {} -> {} candidate steps:", idx, steps.len());
            for s in &steps {
                let cond_names: Vec<String> = s.condition.as_ref()
                    .map(|preds| preds.iter().map(|p| p.name().to_string()).collect())
                    .unwrap_or_default();
                eprintln!("[DIAG]     transformation={:?} conditions={:?} target_spec={:?}", s.transformation, cond_names, s.target_spec);
            }
        }
        if steps.is_empty() {
            if diag { eprintln!("[DIAG] pair {} had ZERO candidates -> generate_common_steps returns empty", idx); }
            return vec![];
        }
        all_candidates.push(steps);
    }

    // Az elso par jelolt-alairasai lesznek a kiindulo halmaz (dedupolva).
    let mut pair0_by_sig: StdHashMap<(String, Vec<String>), &SemanticStep> = StdHashMap::new();
    for step in &all_candidates[0] {
        pair0_by_sig.entry(step_signature(step)).or_insert(step);
    }

    let mut common_steps: Vec<SemanticStep> = Vec::new();

    'sig_loop: for (sig, representative) in &pair0_by_sig {
        let mut steps_per_pair: Vec<&SemanticStep> = vec![*representative];

        for (pair_idx, other_candidates) in all_candidates[1..].iter().enumerate() {
            match other_candidates.iter().find(|s| &step_signature(s) == sig) {
                Some(found) => steps_per_pair.push(found),
                None => {
                    if diag {
                        eprintln!("[DIAG] signature {:?} NOT found in pair {} -> skipped", sig, pair_idx + 1);
                    }
                    continue 'sig_loop;
                }
            }
        }

        if !constant_target_matches(&steps_per_pair) {
            if diag {
                eprintln!("[DIAG] signature {:?} has INCONSISTENT constant target across pairs -> skipped", sig);
            }
            continue 'sig_loop;
        }

        let condition_preds = representative.condition.as_ref().unwrap();
        let pred: Box<dyn Predicate> = if condition_preds.len() == 1 {
            condition_preds[0].clone_box()
        } else {
            Box::new(crate::predicate::builtin::AndPredicate {
                predicates: condition_preds.iter().map(|p| p.clone_box()).collect(),
            })
        };

        let mut unique_all = true;
        for (input, _, _, _) in train_pairs {
            let result = ObjectSelector::select(
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
            if diag { eprintln!("[DIAG] signature {:?} ACCEPTED", sig); }
            common_steps.push((*representative).clone());
        } else if diag {
            eprintln!("[DIAG] signature {:?} matched all pairs but FAILED uniqueness check", sig);
        }
    }

    if diag {
        eprintln!("[DIAG] generate_common_steps returning {} common steps", common_steps.len());
    }
    common_steps
}
