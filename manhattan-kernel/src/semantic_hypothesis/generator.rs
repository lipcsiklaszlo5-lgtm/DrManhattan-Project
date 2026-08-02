use crate::structure::{KernelStructureGraph};
use crate::structure::topology::{graph_diff, NodeTransformation};
use crate::sandbox::operators::Transformation;
use super::semantic_descriptor::describe_node_uniquely;
use super::hypothesis::{SemanticStep, SemanticHypothesis};

/// Generate candidate semantic steps from a single training pair.
/// Returns a list of possible steps (each a pair (predicate_condition, transformation)).
pub fn generate_candidate_steps(
    input: &KernelStructureGraph,
    output: &KernelStructureGraph,
    grid_width: u8,
    grid_height: u8,
) -> Vec<SemanticStep> {
    let diffs = graph_diff(input, output);
    let mut steps = Vec::new();

    for diff in diffs {
        match diff {
            NodeTransformation::Translate { node_id, dx: _, dy: _ } => {
                // Semantic description of the moved object
                if let Some(preds) = describe_node_uniquely(&node_id, input) {
                    // Try to abstract translation to semantic
                    let sem_transform = abstract_translate(&node_id, input, output, grid_width, grid_height);
                    steps.push(SemanticStep {
                        condition: Some(preds),
                        transformation: sem_transform,
                        target_spec: None,
                    });
                }
            }
            NodeTransformation::Recolor { node_id, new_color } => {
                if let Some(preds) = describe_node_uniquely(&node_id, input) {
                    steps.push(SemanticStep {
                        condition: Some(preds),
                        transformation: Transformation::RecolorToTarget { node_id: String::new() }, // node_id placeholder
                        target_spec: Some(crate::abstraction::program::TargetSpec::Constant(new_color)),
                    });
                }
            }
            NodeTransformation::Delete { node_id } => {
                if let Some(preds) = describe_node_uniquely(&node_id, input) {
                    steps.push(SemanticStep {
                        condition: Some(preds),
                        transformation: Transformation::Delete { node_id: String::new() },
                        target_spec: None,
                    });
                }
            }
            NodeTransformation::Create { node_id: _, color: _, bbox_x: _, bbox_y: _ } => {
                // For creation, we need to find a reference object to position relative to.
                // Simple fallback: store color constant, but need semantic position. We'll skip for now to avoid leakage.
                // Instead, we can try to find a stable reference object (largest) and compute relative position.
                // This is complex; for now, we'll omit creation steps to keep purity, but that loses programs.
                // Temporary: we'll generate a step with a condition describing "the object that will be created" impossible, so skip.
            }
            _ => {}
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
    let node_in = match input.nodes.iter().find(|n| n.id == node_id) {
        Some(n) => n,
        None => return Transformation::TranslateToTarget { node_id: node_id.to_string() }, // fallback (impure)
    };
    let node_out = match output.nodes.iter().find(|n| n.id == node_id) {
        Some(n) => n,
        None => return Transformation::TranslateToTarget { node_id: node_id.to_string() },
    };

    let bx: i64 = node_in.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
    let by: i64 = node_in.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
    let bw: u8 = node_in.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1);
    let bh: u8 = node_in.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1);
    let ax: i64 = node_out.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
    let ay: i64 = node_out.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);

    // Check mirror horizontal: after x + w == grid_width - before x
    let mirror_h = (ax + bw as i64) == (grid_width as i64 - bx);
    if mirror_h {
        return Transformation::MirrorHorizontal { node_id: node_id.to_string() };
    }

    // Mirror vertical
    let mirror_v = (ay + bh as i64) == (grid_height as i64 - by);
    if mirror_v {
        return Transformation::MirrorVertical { node_id: node_id.to_string() };
    }

    // Align to corner/center
    let tol = 1;
    let is_top = ay <= tol;
    let is_bottom = (ay + bh as i64) >= (grid_height as i64 - tol);
    let is_left = ax <= tol;
    let is_right = (ax + bw as i64) >= (grid_width as i64 - tol);
    let h_center = (ax + bw as i64/2 - grid_width as i64/2).abs() <= tol;
    let v_center = (ay + bh as i64/2 - grid_height as i64/2).abs() <= tol;

    if is_top && is_left {
        return Transformation::TranslateToTarget { node_id: node_id.to_string() };
        // In future, target_spec will be GridAnchor::TopLeft
    } else if is_top && is_right {
        return Transformation::TranslateToTarget { node_id: node_id.to_string() };
    } else if is_bottom && is_left {
        return Transformation::TranslateToTarget { node_id: node_id.to_string() };
    } else if is_bottom && is_right {
        return Transformation::TranslateToTarget { node_id: node_id.to_string() };
    } else if h_center && v_center {
        return Transformation::TranslateToTarget { node_id: node_id.to_string() };
    }

    // Fallback: keep relative to a reference object (largest stable)
    // For now, we return TranslateToTarget with empty node_id, which will be replaced during compilation.
    Transformation::TranslateToTarget { node_id: String::new() }
}
