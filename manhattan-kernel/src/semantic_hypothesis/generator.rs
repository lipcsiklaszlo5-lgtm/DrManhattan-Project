use crate::structure::KernelStructureGraph;
use crate::structure::topology::{graph_diff, NodeTransformation};
use crate::sandbox::operators::Transformation;
use super::semantic_descriptor::describe_node_all;
use super::hypothesis::{SemanticStep};
use crate::object_selector::ObjectSelector;
use crate::predicate::Predicate;
use crate::abstraction::program::{TargetSpec, GridCorner};
use crate::abstraction::transform::Condition;
use std::collections::HashMap as StdHashMap;

/// Ha a node vegso (output-beli) pozicioja PONTOSAN a racs valamelyik
/// sarkara esik, visszaadja a megfelelo GridAnchor target_spec-et.
/// Ha nem esik pontosan sarokra, None -- ilyenkor NEM generalunk lepest
/// erre a diff-re, mert nincs meg olyan altalanos eszkozunk (pl. relativ
/// pozicionalas egy masik objektumhoz kepest), ami ezt helyesen le tudna
/// irni. Egy None target_spec-es lepes generalasa hamis biztonsagot adna
/// (a lepes soha nem hajtana vegre valos mozgatast).
fn grid_anchor_for_node(
    node_out: &crate::structure::Node,
    grid_width: u8,
    grid_height: u8,
) -> Option<TargetSpec> {
    let diag = std::env::var("MK_DIAG").is_ok();
    let ax: i64 = node_out.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
    let ay: i64 = node_out.attributes.get("bbox_y").and_then(|v| v.parse().ok())?;
    let bw: i64 = node_out.attributes.get("bbox_w").and_then(|v| v.parse().ok())?;
    let bh: i64 = node_out.attributes.get("bbox_h").and_then(|v| v.parse().ok())?;
    let gw = grid_width as i64;
    let gh = grid_height as i64;

    if diag {
        eprintln!("[DIAG-GA] node bbox=({},{},{},{}) grid=({},{})", ax, ay, bw, bh, gw, gh);
    }

    let at_left = ax == 0;
    let at_top = ay == 0;
    let at_right = (ax + bw) == gw;
    let at_bottom = (ay + bh) == gh;

    if diag {
        eprintln!("[DIAG-GA]   at_left={} at_top={} at_right={} at_bottom={}", at_left, at_top, at_right, at_bottom);
    }

    if at_top && at_left {
        Some(TargetSpec::GridAnchor { corner: GridCorner::TopLeft })
    } else if at_top && at_right {
        Some(TargetSpec::GridAnchor { corner: GridCorner::TopRight })
    } else if at_bottom && at_left {
        Some(TargetSpec::GridAnchor { corner: GridCorner::BottomLeft })
    } else if at_bottom && at_right {
        Some(TargetSpec::GridAnchor { corner: GridCorner::BottomRight })
    } else {
        None
    }
}

pub fn generate_candidate_steps(
    input: &KernelStructureGraph,
    output: &KernelStructureGraph,
    grid_width: u8,
    grid_height: u8,
) -> Vec<SemanticStep> {
    let diffs = graph_diff(input, output);
    let mut steps = Vec::new();

    for diff in diffs {
        let node_id = match &diff {
            NodeTransformation::Translate { node_id, .. } => node_id,
            NodeTransformation::Recolor { node_id, .. } => node_id,
            NodeTransformation::Delete { node_id } => node_id,
            _ => continue,
        };

        let (sem_transform, target_spec): (Transformation, Option<TargetSpec>) = match &diff {
            NodeTransformation::Translate { .. } => {
                let mirror_or_translate = abstract_translate(node_id, input, output, grid_width, grid_height);
                match mirror_or_translate {
                    Transformation::SemanticMirrorHorizontal | Transformation::SemanticMirrorVertical => {
                        (mirror_or_translate, None)
                    }
                    _ => {
                        // Csak akkor generalunk lepest, ha a celpozicio egy
                        // racs-sarokra esik pontosan -- egyeb esetben nincs
                        // meg altalanos eszkozunk a celpozicio leirasara.
                        let node_out = match output.nodes.iter().find(|n| n.id == *node_id) {
                            Some(n) => n,
                            None => continue,
                        };
                        match grid_anchor_for_node(node_out, grid_width, grid_height) {
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

                                        // Clone ref_pred for multiple uses
                                        let ref_pred_clone = ref_pred.clone_box();

                                        // 1. Create RelativeToNode step
                                        let relative_spec = TargetSpec::RelativeToNode {
                                            condition: Box::new(Condition::Predicate(ref_pred)),
                                            dx_offset: rel_dx,
                                            dy_offset: rel_dy,
                                        };

                                        // 2. Try Gravitate detection
                                        let gravitate_dx = ax - ref_x;
                                        let gravitate_dy = ay - ref_y;
                                        let is_gravitate = (gravitate_dx == 0 && gravitate_dy != 0) || (gravitate_dy == 0 && gravitate_dx != 0);
                                        if is_gravitate {
                                            let touching = (ax + node_out.attributes.get("bbox_w").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0) == ref_x)
                                                || (ax == ref_x + ref_node.attributes.get("bbox_w").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0))
                                                || (ay + node_out.attributes.get("bbox_h").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0) == ref_y)
                                                || (ay == ref_y + ref_node.attributes.get("bbox_h").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0));
                                            if touching {
                                                let gravitate_spec = TargetSpec::GravitateAnchor {
                                                    anchor_predicate: Box::new(Condition::Predicate(ref_pred_clone)),
                                                };
                                                (Transformation::SemanticGravitate, Some(gravitate_spec))
                                            } else {
                                                (Transformation::SemanticTranslateToTarget, Some(relative_spec))
                                            }
                                        } else {
                                            (Transformation::SemanticTranslateToTarget, Some(relative_spec))
                                        }
                                    } else {
                                        continue
                                    }
                                } else {
                                    continue
                                }
                            }
                        }
                    }
                }
            }
            NodeTransformation::Recolor { new_color, .. } => {
                (Transformation::SemanticRecolorToTarget, Some(TargetSpec::Constant(new_color.clone())))
            }
            NodeTransformation::Delete { .. } => {
                (Transformation::Delete { node_id: String::new() }, None)
            }
            _ => continue,
        };

        let all_descriptions = describe_node_all(node_id, input);
        for preds in all_descriptions {
            steps.push(SemanticStep {
                condition: Some(preds),
                transformation: sem_transform.clone(),
                target_spec: target_spec.clone(),
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

/// Egy jelolt-lepes "alairasa": a transzformacio ALAKJA, a feltetel-nevek
/// rendezett halmaza, ES a target_spec FAJTAJA (nem a konkret erteke -- azt
/// a constant_target_matches ellenorzi kulon). Igy pl. ket
/// SemanticTranslateToTarget lepes csak akkor szamit "ugyanannak", ha
/// mindketto pl. GridAnchor tipusu -- egy GridAnchor es egy semmilyen
/// target_spec nelkuli lepes nem keveredik ossze.

/// Infer a spatial relation between a moved object and its reference.
/// Returns the relation name if the bbox alignment is unambiguous.
fn infer_spatial_relation(
    node: &crate::structure::Node,
    ref_node: &crate::structure::Node,
) -> Option<String> {
    let nx: i64 = node.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
    let ny: i64 = node.attributes.get("bbox_y").and_then(|v| v.parse().ok())?;
    let nw: i64 = node.attributes.get("bbox_w").and_then(|v| v.parse().ok())?;
    let nh: i64 = node.attributes.get("bbox_h").and_then(|v| v.parse().ok())?;
    let rx: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
    let ry: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok())?;
    let rw: i64 = ref_node.attributes.get("bbox_w").and_then(|v| v.parse().ok())?;
    let rh: i64 = ref_node.attributes.get("bbox_h").and_then(|v| v.parse().ok())?;

    let tol = 1i64; // pixel tolerance

    // Vertical relations
    let node_bottom = ny + nh;
    let node_top = ny;
    let ref_bottom = ry + rh;
    let ref_top = ry;
    let node_center_x = nx + nw/2;
    let ref_center_x = rx + rw/2;
    let h_aligned = (node_center_x - ref_center_x).abs() <= tol;

    if node_bottom <= ref_top && h_aligned {
        return Some("Above".to_string());
    }
    if node_top >= ref_bottom && h_aligned {
        return Some("Below".to_string());
    }
    if node_bottom == ref_top {
        return Some("TouchingNorth".to_string());
    }
    if node_top == ref_bottom {
        return Some("TouchingSouth".to_string());
    }

    // Horizontal relations
    let node_right = nx + nw;
    let node_left = nx;
    let ref_right = rx + rw;
    let ref_left = rx;
    let node_center_y = ny + nh/2;
    let ref_center_y = ry + rh/2;
    let v_aligned = (node_center_y - ref_center_y).abs() <= tol;

    if node_right <= ref_left && v_aligned {
        return Some("LeftOf".to_string());
    }
    if node_left >= ref_right && v_aligned {
        return Some("RightOf".to_string());
    }
    if node_right == ref_left {
        return Some("TouchingWest".to_string());
    }
    if node_left == ref_right {
        return Some("TouchingEast".to_string());
    }

    // Alignment relations
    if node_top == ref_top && h_aligned {
        return Some("AlignTop".to_string());
    }
    if node_bottom == ref_bottom && h_aligned {
        return Some("AlignBottom".to_string());
    }
    if node_left == ref_left && v_aligned {
        return Some("AlignLeft".to_string());
    }
    if node_right == ref_right && v_aligned {
        return Some("AlignRight".to_string());
    }

    // Center inside
    if nx >= rx && ny >= ry && (nx + nw) <= (rx + rw) && (ny + nh) <= (ry + rh) {
        return Some("CenterInside".to_string());
    }

    None
}

fn step_signature(step: &SemanticStep) -> (String, Vec<String>, String) {
    let transformation_shape = format!("{:?}", step.transformation);
    let mut cond_names: Vec<String> = step.condition.as_ref()
        .map(|preds| preds.iter().map(|p| p.name().to_string()).collect())
        .unwrap_or_default();
    cond_names.sort();
    let target_kind = match &step.target_spec {
        None => "None".to_string(),
        Some(TargetSpec::Constant(_)) => "Constant".to_string(),
        Some(TargetSpec::GridAnchor { corner }) => format!("GridAnchor:{:?}", corner),
        Some(TargetSpec::RelativeToNode { condition, dx_offset: _, dy_offset: _ }) => {
            // Ha van felismert reláció, azt használjuk a szignatúrában
            format!("RelativeToNode:{}", condition.name())
        }
        Some(TargetSpec::CopyAttributeFrom { .. }) => "CopyAttributeFrom".to_string(),
        Some(TargetSpec::GravitateAnchor { anchor_predicate }) => format!("GravitateAnchor:{}", anchor_predicate.name()),
    };
    (transformation_shape, cond_names, target_kind)
}

/// Ha a lepes target_spec-je Constant(...), az ertekenek meg kell egyeznie
/// MINDEN parban. A GridAnchor mar a step_signature resze (a sarok maga is
/// a signature-be van kodolva), tehat ha a signature-k egyeznek, a sarok is
/// automatikusan egyezik -- itt csak a Constant erteket kell tovabb
/// ellenorizni.
fn constant_target_matches(steps: &[&SemanticStep]) -> bool {
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
}

/// Közös szemantikus lépések generálása több train párból.
/// Halmaz-alapu (nem pozicio-alapu) egyeztetes: minden parra osszegyujtjuk a
/// jelolt lepesek "alairasait" (transzformacio + feltetelek + target_spec
/// fajtaja), majd csak azokat tartjuk meg, amik MINDEN parban elofordulnak,
/// ES a Constant celertek is egyezik mindenhol, ES a feltetel minden parban
/// egyertelmuen (ambiguity=false) egy node-ot valaszt ki.
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

    let mut pair0_by_sig: StdHashMap<(String, Vec<String>, String), &SemanticStep> = StdHashMap::new();
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
