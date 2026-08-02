use manhattan_kernel::object_selector::{ObjectSelector, SelectionStrategy, SelectionResult};
use manhattan_kernel::predicate::builtin::ColorEquals;
use manhattan_kernel::structure::graph::KernelStructureGraph;

#[test]
fn test_object_selector_new() {
    let _selector = ObjectSelector::new();
}

#[test]
fn test_object_selector_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ObjectSelector>();
}

#[test]
fn test_selection_strategy_is_clone_debug() {
    let s = SelectionStrategy::Best;
    let _ = format!("{:?}", s);
    let _ = s.clone();
}

#[test]
fn test_select_with_empty_graph() {
    let selector = ObjectSelector::new();
    let graph = KernelStructureGraph { nodes: vec![], edges: vec![] };
    let pred = ColorEquals::new(1);
    let result = selector.select(&pred, &graph, &SelectionStrategy::All, None);
    assert_eq!(result.selected.len(), 0);
    assert_eq!(result.ambiguity, false);
}
