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
fn test_tie_break_on_equal_area_uses_numeric_id() {
    let g = make_graph(vec![("obj_10", 5, 0, 0, "1"), ("obj_2", 5, 1, 1, "1")]);
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Best, None);
    // obj_2 < obj_10 numerically
    assert_eq!(result.selected[0].node_id, "obj_2");
    assert!(result.ambiguity);
}

#[test]
fn test_tie_break_deterministic_1000_runs() {
    let g = make_graph(vec![("a", 5, 0, 0, "1"), ("b", 5, 1, 1, "1"), ("c", 5, 2, 2, "1")]);
    let first = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Best, None);
    for _ in 0..1000 {
        let next = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Best, None);
        assert_eq!(first.selected[0].node_id, next.selected[0].node_id);
    }
}

#[test]
fn test_1000_iteration_determinism() {
    // Re-run the full determinism suite 1000x
    for _ in 0..1000 {
        test_tie_break_on_equal_area_uses_numeric_id();
    }
}
