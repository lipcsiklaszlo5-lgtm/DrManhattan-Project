use crate::predicate::{Predicate, PredicateResult};
use crate::predicate::builtin;
use crate::structure::{KernelStructureGraph, Node};
use crate::structure::topology::NodeTransformation;
use crate::sandbox::operators::Transformation;

/// Produce a predicate conjunction that uniquely identifies a given node in the graph.
/// Returns the shortest (by specificity) conjunction; if no single predicate is unique,
/// combines predicates via AND.
pub fn describe_node_semantically(
    node_id: &str,
    graph: &KernelStructureGraph,
) -> Option<Box<dyn Predicate>> {
    let target_node = graph.nodes.iter().find(|n| n.id == node_id)?;
    let mut candidates: Vec<Box<dyn Predicate>> = Vec::new();

    // Gather all built-in predicates that can match nodes (single-object predicates).
    // We'll test each predicate and keep those that return exactly the target node.
    let mut preds: Vec<Box<dyn Predicate>> = vec![
        Box::new(builtin::LargestPredicate),
        Box::new(builtin::SmallestPredicate),
        Box::new(builtin::LeftmostPredicate),
        Box::new(builtin::RightmostPredicate),
        Box::new(builtin::TopmostPredicate),
        Box::new(builtin::BottommostPredicate),
        Box::new(builtin::OnlyObjectPredicate),
        Box::new(builtin::UniqueColorPredicate),
        Box::new(builtin::MajorityColorPredicate),
        Box::new(builtin::MinorityColorPredicate),
        Box::new(builtin::CenterObjectPredicate),
        Box::new(builtin::CornerObjectPredicate),
        Box::new(builtin::BorderObjectPredicate),
        // Additional attribute predicates could be added here
    ];

    // Color predicates for each possible color
    for c in 1..=9 {
        preds.push(Box::new(builtin::ColorPredicate { color: c.to_string() }));
    }

    // Evaluate each predicate
    for pred in preds {
        let result = pred.evaluate(graph);
        match result {
            PredicateResult::RankedList(ids) => {
                // Check if it matches only our node
                if ids.len() == 1 && ids[0].0 == node_id {
                    candidates.push(pred);
                }
            }
            PredicateResult::Bool(true) => {
                // matches all nodes – not unique
            }
            _ => {}
        }
    }

    if candidates.is_empty() {
        // If no single predicate unique, try conjunction of two (simple approach)
        // We'll just pick the first two that together narrow down to one node.
        // For simplicity, we can fallback to using the object's color + largest/smallest etc.
        let color = target_node.attributes.get("color").cloned()?;
        let color_pred = builtin::ColorPredicate { color };
        let largest = builtin::LargestPredicate;
        // Combine AND
        let combined = builtin::AndPredicate {
            predicates: vec![Box::new(color_pred), Box::new(largest)],
        };
        // Verify uniqueness
        match combined.evaluate(graph) {
            PredicateResult::RankedList(ids) if ids.len() == 1 && ids[0].0 == node_id => {
                return Some(Box::new(combined));
            }
            _ => return None,
        }
    }

    // Choose the "best" (highest specificity, i.e. most specific)
    candidates.sort_by_key(|p| p.specificity());
    Some(candidates.into_iter().next_back().unwrap())
}

/// Convert a graph diff transformation into a semantic transformation.
/// Returns None if the transformation cannot be generalized (e.g., pure creation/deletion).
pub fn generalize_transformation(
    diff: &NodeTransformation,
    before: &KernelStructureGraph,
    after: &KernelStructureGraph,
    grid_width: u8,
    grid_height: u8,
) -> Option<(Option<Box<dyn Predicate>>, Transformation)> {
    match diff {
        NodeTransformation::Translate { node_id, dx: _, dy: _ } => {
            // Try to infer semantic translation
            let semantic_cond = describe_node_semantically(node_id, before)?;
            let node_before = before.nodes.iter().find(|n| n.id == *node_id)?;
            let node_after = after.nodes.iter().find(|n| n.id == *node_id)?;
            let bx: i64 = node_before.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
            let by: i64 = node_before.attributes.get("bbox_y").and_then(|v| v.parse().ok())?;
            let bw: u8 = node_before.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1);
            let bh: u8 = node_before.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1);
            let ax: i64 = node_after.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
            let ay: i64 = node_after.attributes.get("bbox_y").and_then(|v| v.parse().ok())?;

            // Check if moved to a grid anchor (corner, edge center)
            let tolerance = 1; // pixels
            let is_top = ay <= tolerance;
            let is_bottom = (ay + bh as i64) >= (grid_height as i64 - tolerance);
            let is_left = ax <= tolerance;
            let is_right = (ax + bw as i64) >= (grid_width as i64 - tolerance);
            let is_h_center = (ax + bw as i64 / 2 - grid_width as i64 / 2).abs() <= tolerance;
            let is_v_center = (ay + bh as i64 / 2 - grid_height as i64 / 2).abs() <= tolerance;

            if is_top && is_left {
                return Some((Some(semantic_cond), Transformation::TranslateToTarget {
                    node_id: node_id.clone(),
                }));
                // We'll use TargetSpec::GridAnchor { corner: GridCorner::TopLeft } in later compilation
            } else if is_top && is_right {
                return Some((Some(semantic_cond), Transformation::TranslateToTarget { node_id: node_id.clone() }));
            } else if is_bottom && is_left {
                return Some((Some(semantic_cond), Transformation::TranslateToTarget { node_id: node_id.clone() }));
            } else if is_bottom && is_right {
                return Some((Some(semantic_cond), Transformation::TranslateToTarget { node_id: node_id.clone() }));
            } else if is_h_center && is_v_center {
                return Some((Some(semantic_cond), Transformation::TranslateToTarget { node_id: node_id.clone() }));
            } else {
                // Fallback: TranslateToTarget with RelativeToNode? We need a reference object.
                // For now, we'll keep a Translate but with empty node_id (to be replaced by condition).
                // The absolute dx,dy will be stored in the transformation, but we avoid node_id.
                // To satisfy purity, we must avoid dx,dy as well. We'll use TranslateToTarget with a GridAnchor that is not exactly corner but relative? Not good.
                // Alternative: we'll add a new variant TranslateRelative { dx, dy }? Still numeric.
                // As a temporary measure, we'll keep the numeric dx,dy but store them as part of a target_spec that references the grid itself? The requirement is strict: no absolute coordinates. So we need to eliminate dx,dy completely.
                // We can encode the movement as a MirrorHorizontal if dx + bw == grid_width - ax? That would be semantic.
                // Check mirror horizontal: the object's x + width after should be symmetric to before relative to center.
                let mirror_h = (ax + bw as i64) == (grid_width as i64 - bx) || (ax == grid_width as i64 - (bx + bw as i64));
                if mirror_h {
                    return Some((Some(semantic_cond), Transformation::MirrorHorizontal { node_id: node_id.clone() }));
                }
                let mirror_v = (ay + bh as i64) == (grid_height as i64 - by) || (ay == grid_height as i64 - (by + bh as i64));
                if mirror_v {
                    return Some((Some(semantic_cond), Transformation::MirrorVertical { node_id: node_id.clone() }));
                }
                // If no semantic match, fallback to TranslateToTarget with RelativeToNode using a reference object (e.g., the largest static object). This still uses dx,dy relative to that object, which are not absolute grid coordinates, but still numeric. We'll accept that as improvement.
                let ref_node = find_stable_reference(before, node_id)?;
                let ref_cond = describe_node_semantically(&ref_node.id, before)?;
                let ref_x: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
                let ref_y: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok())?;
                let rel_dx = ax - ref_x;
                let rel_dy = ay - ref_y;
                // Store as RelativeToNode in target_spec later; for now we keep transformation as TranslateToTarget with placeholder, and we'll set target_spec in the caller.
                return Some((Some(semantic_cond), Transformation::TranslateToTarget { node_id: node_id.clone() }));
            }
        }
        NodeTransformation::Recolor { node_id, new_color } => {
            let semantic_cond = describe_node_semantically(node_id, before)?;
            Some((Some(semantic_cond), Transformation::RecolorToTarget { node_id: node_id.clone() }))
        }
        NodeTransformation::Delete { node_id } => {
            let semantic_cond = describe_node_semantically(node_id, before)?;
            Some((Some(semantic_cond), Transformation::Delete { node_id: node_id.clone() }))
        }
        NodeTransformation::Create { node_id, color, bbox_x: _, bbox_y: _ } => {
            // Creation: we cannot describe the node before (it doesn't exist), but we can describe its position relative to something.
            // For now, we'll skip condition and use a direct Create with absolute coords? Not allowed.
            // We'll encode creation as "Create at position relative to some reference", using target_spec.
            None // can't fully generalize creation yet; return None and let caller fallback to concrete? But that would leak.
        }
        _ => None,
    }
}

fn find_stable_reference(graph: &KernelStructureGraph, exclude_id: &str) -> Option<Node> {
    // Pick the largest object that is not the moving one.
    graph.nodes.iter()
        .filter(|n| n.id != exclude_id)
        .max_by_key(|n| {
            n.attributes.get("area").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0)
        })
        .cloned()
}
