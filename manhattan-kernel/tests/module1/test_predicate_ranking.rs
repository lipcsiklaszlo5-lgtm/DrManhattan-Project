use manhattan_kernel::predicate::{Predicate, PredicateResult};
use manhattan_kernel::predicate::builtin::*;
use manhattan_kernel::structure::KernelStructureGraph;

#[test]
fn test_ranking_order() {
    let mut g = KernelStructureGraph::new();
    let a = g.add_node("a", "arc_object");
    a.attributes.insert("area".into(), "3".into());
    a.attributes.insert("bbox_x".into(), "0".into());
    a.attributes.insert("bbox_y".into(), "0".into());
    a.attributes.insert("bbox_w".into(), "1".into());
    a.attributes.insert("bbox_h".into(), "1".into());

    let b = g.add_node("b", "arc_object");
    b.attributes.insert("area".into(), "8".into());
    b.attributes.insert("bbox_x".into(), "10".into());
    b.attributes.insert("bbox_y".into(), "10".into());
    b.attributes.insert("bbox_w".into(), "1".into());
    b.attributes.insert("bbox_h".into(), "1".into());

    let c = g.add_node("c", "arc_object");
    c.attributes.insert("area".into(), "5".into());
    c.attributes.insert("bbox_x".into(), "0".into());
    c.attributes.insert("bbox_y".into(), "1".into());
    c.attributes.insert("bbox_w".into(), "1".into());
    c.attributes.insert("bbox_h".into(), "1".into());

    // Largest
    if let PredicateResult::RankedList(list) = LargestPredicate.evaluate(&g) {
        assert_eq!(list[0].0, "b");
    } else { panic!("Expected RankedList"); }

    // Smallest
    if let PredicateResult::RankedList(list) = SmallestPredicate.evaluate(&g) {
        assert_eq!(list[0].0, "a");
    } else { panic!("Expected RankedList"); }

    // Nearest to a (0,0)
    if let PredicateResult::RankedList(list) = NearestPredicate {
        reference: Box::new(ColorPredicate { color: "0".into() }),
    }.evaluate(&g) {
        // Ha nincs color=0, akkor a reference üres -> false
        assert_eq!(list.len(), 0); // vagy false
    }

    // Leftmost
    if let PredicateResult::RankedList(list) = LeftmostPredicate.evaluate(&g) {
        assert!(list.iter().any(|(id, _)| id == "a" || id == "c"));
    }
}
