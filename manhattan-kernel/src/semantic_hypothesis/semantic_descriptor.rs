use crate::predicate::{Predicate, PredicateResult};
use crate::predicate::builtin;
use crate::structure::KernelStructureGraph;

/// Generate a canonical semantic description (predicate conjunction) that uniquely
/// identifies the given node in the graph.
/// Returns the conjunction as a vector of predicates (AND semantics), or None if impossible.
pub fn describe_node_uniquely(
    node_id: &str,
    graph: &KernelStructureGraph,
) -> Option<Vec<Box<dyn Predicate>>> {
    let target_node = graph.nodes.iter().find(|n| n.id == node_id)?;
    let mut candidate_preds: Vec<Box<dyn Predicate>> = Vec::new();

    // Built-in predicates
    let builtins: Vec<Box<dyn Predicate>> = vec![
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
    ];

    for pred in builtins {
        let result = pred.evaluate(graph);
        if let PredicateResult::RankedList(ids) = result {
            if ids.len() == 1 && ids[0].0 == node_id {
                candidate_preds.push(pred);
            }
        }
    }

    // Color predicates
    for c in 1..=9 {
        let color_pred = builtin::ColorPredicate { color: c.to_string() };
        let result = color_pred.evaluate(graph);
        if let PredicateResult::RankedList(ids) = result {
            if ids.len() == 1 && ids[0].0 == node_id {
                candidate_preds.push(Box::new(color_pred));
            }
        }
    }

    if candidate_preds.is_empty() {
        // Fallback: combine color and largest to try to get unique
        let color = target_node.attributes.get("color")?.clone();
        let cp = builtin::ColorPredicate { color };
        let lp = builtin::LargestPredicate;
        let combined = builtin::AndPredicate {
            predicates: vec![Box::new(cp), Box::new(lp)],
        };
        if let PredicateResult::RankedList(ids) = combined.evaluate(graph) {
            if ids.len() == 1 && ids[0].0 == node_id {
                return Some(vec![Box::new(combined)]);
            }
        }
        return None;
    }

    // Pick the most specific single predicate if it uniquely identifies; otherwise combine.
    candidate_preds.sort_by_key(|p| p.specificity());
    // Try the most specific first
    let best = candidate_preds.last()?;
    let result = best.evaluate(graph);
    if let PredicateResult::RankedList(ids) = result {
        if ids.len() == 1 && ids[0].0 == node_id {
            return Some(vec![best.clone_box()]);
        }
    }
    // Not unique with one; try combination of the two most specific that together give uniqueness
    if candidate_preds.len() >= 2 {
        let p1 = candidate_preds[candidate_preds.len()-1].clone_box();
        let p2 = candidate_preds[candidate_preds.len()-2].clone_box();
        let combined = builtin::AndPredicate {
            predicates: vec![p1, p2],
        };
        if let PredicateResult::RankedList(ids) = combined.evaluate(graph) {
            if ids.len() == 1 && ids[0].0 == node_id {
                return Some(vec![Box::new(combined)]);
            }
        }
    }
    None
}

/// Generate ALL semantic descriptions (predicate conjunctions) that uniquely
/// identify the given node in the graph -- not just the first/best one.
/// Each element of the returned Vec is one alternative AND-combination of
/// predicates that, together, select exactly this node. This gives the
/// generator multiple candidates to try across train pairs, increasing the
/// chance that at least one description generalizes to all of them.
pub fn describe_node_all(
    node_id: &str,
    graph: &KernelStructureGraph,
) -> Vec<Vec<Box<dyn Predicate>>> {
    let target_node = match graph.nodes.iter().find(|n| n.id == node_id) {
        Some(n) => n,
        None => return Vec::new(),
    };

    let mut results: Vec<Vec<Box<dyn Predicate>>> = Vec::new();

    let builtins: Vec<Box<dyn Predicate>> = vec![
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
    ];

    let mut unique_singles: Vec<Box<dyn Predicate>> = Vec::new();

    for pred in builtins {
        if let PredicateResult::RankedList(ids) = pred.evaluate(graph) {
            if ids.len() == 1 && ids[0].0 == node_id {
                unique_singles.push(pred);
            }
        }
    }

    for c in 1..=9 {
        let color_pred = builtin::ColorPredicate { color: c.to_string() };
        if let PredicateResult::RankedList(ids) = color_pred.evaluate(graph) {
            if ids.len() == 1 && ids[0].0 == node_id {
                unique_singles.push(Box::new(color_pred));
            }
        }
    }

    // Minden onmagaban egyedien azonosito predikatum kulon jelolt.
    for pred in &unique_singles {
        results.push(vec![pred.clone_box()]);
    }

    // Parok kombinacioja is jelolt (akkor is, ha egyenkent mar egyedik --
    // igy tobb, egymastol fuggetlen leiras all rendelkezesre a generalizacioz).
    for i in 0..unique_singles.len() {
        for j in (i + 1)..unique_singles.len() {
            let combined = builtin::AndPredicate {
                predicates: vec![unique_singles[i].clone_box(), unique_singles[j].clone_box()],
            };
            if let PredicateResult::RankedList(ids) = combined.evaluate(graph) {
                if ids.len() == 1 && ids[0].0 == node_id {
                    results.push(vec![Box::new(combined)]);
                }
            }
        }
    }

    // Fallback: ha semmi nem egyedi onmagaban, probaljuk a szin + legnagyobb
    // kombinaciot, mint az eredeti describe_node_uniquely fallback aga.
    if results.is_empty() {
        if let Some(color) = target_node.attributes.get("color").cloned() {
            let cp = builtin::ColorPredicate { color };
            let lp = builtin::LargestPredicate;
            let combined = builtin::AndPredicate {
                predicates: vec![Box::new(cp), Box::new(lp)],
            };
            if let PredicateResult::RankedList(ids) = combined.evaluate(graph) {
                if ids.len() == 1 && ids[0].0 == node_id {
                    results.push(vec![Box::new(combined)]);
                }
            }
        }
    }

    results
}

/// Canonicalize a list of predicates (order, normalization) not yet implemented fully.
pub fn canonicalize(predicates: Vec<Box<dyn Predicate>>) -> Vec<Box<dyn Predicate>> {
    // Sort by name for now
    let mut preds: Vec<_> = predicates.into_iter().collect();
    preds.sort_by(|a, b| a.name().cmp(b.name()));
    preds
}
