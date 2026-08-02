use manhattan_kernel::predicate::{Predicate, PredicateResult};
use manhattan_kernel::predicate::builtin::*;
use manhattan_kernel::structure::KernelStructureGraph;

#[test]
fn test_deeply_nested_and_or_not() {
    let mut g = KernelStructureGraph::new();
    let a = g.add_node("a", "arc_object");
    a.attributes.insert("color".into(), "1".into());
    a.attributes.insert("area".into(), "5".into());
    let b = g.add_node("b", "arc_object");
    b.attributes.insert("color".into(), "2".into());
    b.attributes.insert("area".into(), "3".into());

    // NOT(AND(color=1, area=5)) should return node b
    let nested = NotPredicate {
        predicate: Box::new(AndPredicate {
            predicates: vec![
                Box::new(ColorPredicate { color: "1".into() }),
                Box::new(AreaPredicate { min: Some(5), max: Some(5) }),
            ],
        }),
    };
    let result = nested.evaluate(&g);
    if let PredicateResult::RankedList(list) = result {
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "b");
    } else { panic!("Expected RankedList"); }
}

#[test]
fn test_or_predicate_union() {
    let mut g = KernelStructureGraph::new();
    let a = g.add_node("a", "arc_object");
    a.attributes.insert("color".into(), "1".into());
    let b = g.add_node("b", "arc_object");
    b.attributes.insert("color".into(), "2".into());

    let or = OrPredicate {
        predicates: vec![
            Box::new(ColorPredicate { color: "1".into() }),
            Box::new(ColorPredicate { color: "2".into() }),
        ],
    };
    let result = or.evaluate(&g);
    if let PredicateResult::RankedList(list) = result {
        assert_eq!(list.len(), 2);
    } else { panic!("Expected RankedList"); }
}

#[test]
fn test_empty_graph_handling() {
    let g = KernelStructureGraph::new();
    let pred = ColorPredicate { color: "1".into() };
    assert_eq!(pred.evaluate(&g), PredicateResult::Bool(false));

    let largest = LargestPredicate.evaluate(&g);
    assert_eq!(largest, PredicateResult::Bool(false));

    let and = AndPredicate {
        predicates: vec![
            Box::new(ColorPredicate { color: "1".into() }),
            Box::new(LargestPredicate),
        ],
    };
    assert_eq!(and.evaluate(&g), PredicateResult::Bool(false));
}
