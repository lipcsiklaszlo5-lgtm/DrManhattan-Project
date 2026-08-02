use manhattan_kernel::abstraction::program::{GeneralizedProgram, AbstractStep, Cardinality};
use manhattan_kernel::predicate::builtin::ColorEquals;
use manhattan_kernel::sandbox::operators::Transformation;
use manhattan_kernel::structure::graph::KernelStructureGraph;
use manhattan_kernel::object_selector::ObjectSelector;

#[test]
fn test_matching_nodes_still_exists() {
    // GeneralizedProgram should have matching_nodes method or equivalent
    let step = AbstractStep {
        condition: Some(Box::new(ColorEquals::new(1))),
        transformation: Transformation::NoOp,
        target_spec: None,
        cardinality: Cardinality::All,
    };
    let prog = GeneralizedProgram {
        steps: vec![step],
        confidence: 1.0,
        num_train_pairs: 0,
    };
    let graph = KernelStructureGraph { nodes: vec![], edges: vec![] };
    // This call should compile and not panic
    let _ = prog.matching_nodes(&graph, 0);
}

#[test]
fn test_legacy_all_maps_to_object_selector() {
    // Verify ObjectSelector::select with All produces same result as iterating all nodes
    let selector = ObjectSelector::new();
    let graph = KernelStructureGraph { nodes: vec![], edges: vec![] };
    let pred = ColorEquals::new(1);
    let result = selector.select(&pred, &graph, &manhattan_kernel::object_selector::SelectionStrategy::All, None);
    assert_eq!(result.selected.len(), 0);
}
