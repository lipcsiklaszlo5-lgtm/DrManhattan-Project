use manhattan_kernel::object_selector::{ObjectSelector, SelectionStrategy};
use manhattan_kernel::predicate::builtin::*;
use manhattan_kernel::structure::KernelStructureGraph;

fn make_graph(specs: Vec<(&str, u64, i64, i64, &str)>) -> KernelStructureGraph {
    let mut g = KernelStructureGraph::new();
    for (id, area, x, y, color) in specs {
        let node = g.add_node(id, "arc_object");
        node.attributes.insert("area".into(), area.to_string());
        node.attributes.insert("bbox_x".into(), x.to_string());
        node.attributes.insert("bbox_y".into(), y.to_string());
        node.attributes.insert("bbox_w".into(), "1".to_string());
        node.attributes.insert("bbox_h".into(), "1".to_string());
        node.attributes.insert("color".into(), color.to_string());
    }
    g
}

#[test]
fn test_topk_returns_k_elements() {
    let g = make_graph(vec![("a", 1, 0, 0, "1"), ("b", 3, 1, 1, "1"), ("c", 5, 2, 2, "1"), ("d", 7, 3, 3, "1")]);
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::TopK(2), None);
    assert_eq!(result.selected.len(), 2);
}

#[test]
fn test_topk_correct_order() {
    let g = make_graph(vec![("a", 1, 0, 0, "1"), ("b", 5, 1, 1, "1"), ("c", 3, 2, 2, "1")]);
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::TopK(3), None);
    let ids: Vec<&str> = result.selected.iter().map(|s| s.node_id.as_str()).collect();
    assert_eq!(ids, vec!["b", "c", "a"]);
}

#[test]
fn test_threshold_filters_correctly() {
    let g = make_graph(vec![("a", 1, 0, 0, "1"), ("b", 5, 1, 1, "1"), ("c", 10, 2, 2, "1")]);
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Threshold(0.5), None);
    // All three should pass threshold since Largest gives all positive scores
    assert_eq!(result.selected.len(), 3);
}

#[test]
fn test_threshold_high_value_returns_empty() {
    let g = make_graph(vec![("a", 1, 0, 0, "1")]);
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Threshold(999.0), None);
    assert!(result.selected.is_empty());
}

#[test]
fn test_all_returns_every_candidate() {
    let g = make_graph(vec![("a", 1, 0, 0, "1"), ("b", 5, 1, 1, "1"), ("c", 10, 2, 2, "2")]);
    // ColorPredicate "1" matches two objects
    let pred = ColorPredicate { color: "1".into() };
    let result = ObjectSelector::select(&pred, &g, &SelectionStrategy::All, None);
    assert_eq!(result.selected.len(), 2);
}

#[test]
fn test_unique_returns_ambiguity_when_multiple() {
    let g = make_graph(vec![("a", 5, 0, 0, "1"), ("b", 5, 1, 1, "1")]);
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Unique, None);
    assert!(result.ambiguity);
    assert_eq!(result.selected.len(), 1); // still returns one, but flagged
}

#[test]
fn test_unique_returns_ok_when_exactly_one() {
    let g = make_graph(vec![("only", 5, 0, 0, "1")]);
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Unique, None);
    assert!(!result.ambiguity);
    assert_eq!(result.selected[0].node_id, "only");
}
