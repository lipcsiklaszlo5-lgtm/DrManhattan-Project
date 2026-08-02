use manhattan_kernel::predicate::{Predicate, PredicateResult};
use manhattan_kernel::abstraction::transform::Condition;
use manhattan_kernel::structure::KernelStructureGraph;

#[test]
fn test_condition_always_true() {
    let g = KernelStructureGraph::new();
    let cond = Condition::AlwaysTrue;
    assert_eq!(cond.evaluate(&g), PredicateResult::Bool(true));
}

#[test]
fn test_condition_node_has_attribute() {
    let mut g = KernelStructureGraph::new();
    let n = g.add_node("a", "arc_object");
    n.attributes.insert("color".into(), "1".into());

    let cond = Condition::NodeHasAttribute("color".into(), "1".into());
    if let PredicateResult::RankedList(list) = cond.evaluate(&g) {
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "a");
    } else { panic!("Expected RankedList"); }
}

#[test]
fn test_condition_color_equals() {
    let mut g = KernelStructureGraph::new();
    let n = g.add_node("a", "arc_object");
    n.attributes.insert("color".into(), "1".into());

    let cond = Condition::ColorEquals("1".into());
    if let PredicateResult::RankedList(list) = cond.evaluate(&g) {
        assert_eq!(list.len(), 1);
    } else { panic!("Expected RankedList"); }
}
