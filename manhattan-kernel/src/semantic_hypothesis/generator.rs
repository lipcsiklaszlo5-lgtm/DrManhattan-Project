use crate::structure::{KernelStructureGraph};
use crate::structure::topology::{graph_diff, NodeTransformation};
use crate::sandbox::operators::Transformation;
use super::semantic_descriptor::describe_node_uniquely;
use super::hypothesis::{SemanticStep};

/// Generate candidate semantic steps from a single training pair.
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
                if let Some(preds) = describe_node_uniquely(&node_id, input) {
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
                        transformation: Transformation::SemanticRecolorToTarget,
                        target_spec: Some(crate::abstraction::program::TargetSpec::Constant(new_color)),
                    });
                }
            }
            NodeTransformation::Delete { node_id } => {
                if let Some(preds) = describe_node_uniquely(&node_id, input) {
                    steps.push(SemanticStep {
                        condition: Some(preds),
                        transformation: Transformation::Delete { node_id: String::new() }, // node_id will be filled at execution time from selected nodes
                        target_spec: None,
                    });
                }
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
        None => return Transformation::SemanticTranslateToTarget,
    };
    let node_out = match output.nodes.iter().find(|n| n.id == node_id) {
        Some(n) => n,
        None => return Transformation::SemanticTranslateToTarget,
    };

    let bx: i64 = node_in.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
    let by: i64 = node_in.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
    let bw: u8 = node_in.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1);
    let bh: u8 = node_in.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1);
    let ax: i64 = node_out.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
    let ay: i64 = node_out.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);

    let mirror_h = (ax + bw as i64) == (grid_width as i64 - bx);
    if mirror_h {
        return Transformation::SemanticMirrorHorizontal;
    }

    let mirror_v = (ay + bh as i64) == (grid_height as i64 - by);
    if mirror_v {
        return Transformation::SemanticMirrorVertical;
    }

    // Fallback: use SemanticTranslateToTarget with target_spec to be filled later (we lack target spec here)
    Transformation::SemanticTranslateToTarget
}
