use manhattan_kernel::predicate::{Predicate, PredicateResult};
use manhattan_kernel::predicate::builtin::*;
use manhattan_kernel::structure::KernelStructureGraph;

#[test]
fn test_determinism_largest() {
    let mut g = KernelStructureGraph::new();
    for i in 0..5 {
        let node = g.add_node(&format!("obj{}", i), "arc_object");
        node.attributes.insert("area".into(), (i * 10).to_string());
    }
    let first = LargestPredicate.evaluate(&g);
    for _ in 0..1000 {
        let next = LargestPredicate.evaluate(&g);
        assert_eq!(first, next, "Largest must be deterministic");
    }
}

#[test]
fn test_determinism_ranking_order() {
    let mut g = KernelStructureGraph::new();
    for i in 0..10 {
        let node = g.add_node(&format!("obj{}", i), "arc_object");
        node.attributes.insert("area".into(), (10 - i).to_string());
    }
    let first = LargestPredicate.evaluate(&g);
    for _ in 0..100 {
        let next = LargestPredicate.evaluate(&g);
        assert_eq!(first, next, "Largest ranking must be stable");
    }
}
