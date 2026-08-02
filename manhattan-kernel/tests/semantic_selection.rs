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
fn test_largest_selects_max_area() {
    let g = make_graph(vec![("a", 2, 0, 0, "1"), ("b", 5, 1, 1, "1"), ("c", 10, 2, 2, "1")]);
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Best, None);
    assert_eq!(result.selected[0].node_id, "c");
}

#[test]
fn test_leftmost_selects_min_x() {
    let g = make_graph(vec![("a", 5, 10, 0, "1"), ("b", 5, 3, 5, "1"), ("c", 5, 7, 5, "1")]);
    let result = ObjectSelector::select(&LeftmostPredicate, &g, &SelectionStrategy::Best, None);
    assert_eq!(result.selected[0].node_id, "b");
}

#[test]
fn test_smallest_selects_min_area() {
    let g = make_graph(vec![("a", 2, 0, 0, "1"), ("b", 5, 1, 1, "1"), ("c", 10, 2, 2, "1")]);
    let result = ObjectSelector::select(&SmallestPredicate, &g, &SelectionStrategy::Best, None);
    assert_eq!(result.selected[0].node_id, "a");
}

#[test]
fn test_empty_graph_returns_empty() {
    let g = KernelStructureGraph::new();
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Best, None);
    assert!(result.selected.is_empty());
    assert!(!result.ambiguity);
}

#[test]
fn test_single_object_always_selected() {
    let g = make_graph(vec![("only", 1, 0, 0, "1")]);
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Best, None);
    assert_eq!(result.selected[0].node_id, "only");
    assert!(!result.ambiguity);
}

#[test]
fn test_no_matching_predicate_returns_empty() {
    let g = make_graph(vec![("a", 5, 0, 0, "1")]);
    // ColorPredicate for "2" – no object has color 2
    let pred = ColorPredicate { color: "2".into() };
    let result = ObjectSelector::select(&pred, &g, &SelectionStrategy::Best, None);
    assert!(result.selected.is_empty());
}

#[test]
fn test_border_object_selection() {
    // BorderObject: objects at grid edges (x=0 or y=0 or x=max or y=max)
    let g = make_graph(vec![
        ("inner", 5, 1, 1, "1"),
        ("border_top", 5, 0, 0, "1"),
        ("border_left", 5, 0, 3, "1"),
    ]);
    let result = ObjectSelector::select(&BorderObjectPredicate, &g, &SelectionStrategy::All, None);
    let ids: Vec<&str> = result.selected.iter().map(|s| s.node_id.as_str()).collect();
    assert!(!ids.contains(&"inner"));
    assert!(ids.contains(&"border_top") || ids.contains(&"border_left"));
}
