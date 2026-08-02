use manhattan_kernel::predicate::{Predicate, PredicateResult};
use manhattan_kernel::predicate::builtin::*;
use manhattan_kernel::structure::KernelStructureGraph;

fn build_graph(objects: Vec<(&str, u64, i64, i64, &str)>) -> KernelStructureGraph {
    let mut g = KernelStructureGraph::new();
    for (id, area, x, y, color) in objects {
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
fn test_largest_predicate_semantics() {
    let g = build_graph(vec![
        ("a", 2, 0, 0, "1"),
        ("b", 5, 1, 1, "1"),
        ("c", 9, 2, 2, "2"),
    ]);
    let result = LargestPredicate.evaluate(&g);
    if let PredicateResult::RankedList(list) = result {
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "c", "Largest should return the object with max area");
    } else { panic!("Expected RankedList"); }
}

#[test]
fn test_smallest_predicate_semantics() {
    let g = build_graph(vec![
        ("a", 2, 0, 0, "1"),
        ("b", 5, 1, 1, "1"),
        ("c", 9, 2, 2, "2"),
    ]);
    let result = SmallestPredicate.evaluate(&g);
    if let PredicateResult::RankedList(list) = result {
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "a", "Smallest should return the object with min area");
    } else { panic!("Expected RankedList"); }
}

#[test]
fn test_leftmost_predicate_semantics() {
    let g = build_graph(vec![
        ("a", 2, 5, 0, "1"),
        ("b", 5, 1, 1, "1"),
        ("c", 9, 3, 2, "2"),
    ]);
    let result = LeftmostPredicate.evaluate(&g);
    if let PredicateResult::RankedList(list) = result {
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "b", "Leftmost should return the object with min bbox_x");
    } else { panic!("Expected RankedList"); }
}

#[test]
fn test_border_object_semantics() {
    let g = build_graph(vec![
        ("a", 2, 0, 5, "1"),
        ("b", 5, 1, 0, "1"),
        ("c", 9, 3, 2, "2"),
    ]);
    let result = BorderObjectPredicate.evaluate(&g);
    if let PredicateResult::RankedList(list) = result {
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|(id, _)| id == "a"));
        assert!(list.iter().any(|(id, _)| id == "b"));
    } else { panic!("Expected RankedList"); }
}

#[test]
fn test_hole_inside_contains_semantics() {
    let mut g = KernelStructureGraph::new();
    let a = g.add_node("a", "arc_object");
    a.attributes.insert("area".into(), "10".into());
    let b = g.add_node("b", "arc_object");
    b.attributes.insert("area".into(), "2".into());
    g.add_edge("a", "b", "contains");
    
    let hole = HolePredicate.evaluate(&g);
    if let PredicateResult::RankedList(list) = hole {
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "b", "Hole should be the contained node");
    } else { panic!("Expected RankedList"); }
    
    let inside = InsidePredicate { reference: Box::new(ColorPredicate { color: "0".into() }) };
    let _ = inside.evaluate(&g);
    
    let contains = ContainsPredicate { reference: Box::new(HolePredicate) };
    if let PredicateResult::RankedList(list) = contains.evaluate(&g) {
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "a", "Contains should be the container");
    } else { panic!("Expected RankedList"); }
}

#[test]
fn test_nearest_farthest_semantics() {
    let g = build_graph(vec![
        ("ref", 5, 0, 0, "1"),
        ("near", 3, 1, 1, "2"),
        ("far", 3, 10, 10, "3"),
    ]);
    let nearest = NearestPredicate { reference: Box::new(ColorPredicate { color: "1".into() }) };
    if let PredicateResult::RankedList(list) = nearest.evaluate(&g) {
        assert!(!list.is_empty());
        assert_eq!(list[0].0, "near", "Nearest to ref should be near");
    } else { panic!("Expected RankedList"); }
    
    let farthest = FarthestPredicate { reference: Box::new(ColorPredicate { color: "1".into() }) };
    if let PredicateResult::RankedList(list) = farthest.evaluate(&g) {
        assert!(!list.is_empty());
        assert_eq!(list[0].0, "far", "Farthest from ref should be far");
    } else { panic!("Expected RankedList"); }
}

#[test]
fn test_majority_minority_color_semantics() {
    let g = build_graph(vec![
        ("a", 1, 0, 0, "1"),
        ("b", 1, 1, 1, "1"),
        ("c", 1, 2, 2, "1"),
        ("d", 1, 3, 3, "2"),
    ]);
    let majority = MajorityColorPredicate.evaluate(&g);
    if let PredicateResult::RankedList(list) = majority {
        assert_eq!(list.len(), 3);
        assert!(list.iter().all(|(id, _)| id == "a" || id == "b" || id == "c"));
    } else { panic!("Expected RankedList"); }
    
    let minority = MinorityColorPredicate.evaluate(&g);
    if let PredicateResult::RankedList(list) = minority {
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "d");
    } else { panic!("Expected RankedList"); }
}

#[test]
fn test_composition_and_largest_red() {
    let g = build_graph(vec![
        ("a", 5, 0, 0, "1"),
        ("b", 3, 1, 1, "1"),
        ("c", 9, 2, 2, "2"),
        ("d", 8, 3, 3, "1"),
    ]);
    let and = AndPredicate {
        predicates: vec![
            Box::new(ColorPredicate { color: "1".into() }),
            Box::new(LargestPredicate),
        ],
    };
    if let PredicateResult::RankedList(list) = and.evaluate(&g) {
        assert_eq!(list.len(), 0, "AND Largest Red should be empty because largest (c) is not red");
    } else { panic!("Expected RankedList (maybe empty)"); }
}

#[test]
fn test_composition_nested_not_or() {
    // Különböző színekkel, hogy a SymmetryPredicate ne egyezzen
    let mut g = KernelStructureGraph::new();
    let a = g.add_node("a", "arc_object");
    a.attributes.insert("area".into(), "5".into());
    a.attributes.insert("color".into(), "1".into());
    let b = g.add_node("b", "arc_object");
    b.attributes.insert("area".into(), "3".into());
    b.attributes.insert("color".into(), "2".into());
    g.add_edge("a", "b", "contains");
    
    let not_or = NotPredicate {
        predicate: Box::new(OrPredicate {
            predicates: vec![
                Box::new(HolePredicate),
                Box::new(SymmetryPredicate),
            ],
        }),
    };
    if let PredicateResult::RankedList(list) = not_or.evaluate(&g) {
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "a");
    } else { panic!("Expected RankedList"); }
}

#[test]
fn test_composition_xor() {
    let g = build_graph(vec![
        ("a", 1, 0, 0, "1"),
        ("b", 1, 1, 1, "2"),
        ("c", 1, 2, 2, "3"),
    ]);
    let xor = XorPredicate {
        a: Box::new(ColorPredicate { color: "1".into() }),
        b: Box::new(ColorPredicate { color: "2".into() }),
    };
    if let PredicateResult::RankedList(list) = xor.evaluate(&g) {
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|(id, _)| id == "a"));
        assert!(list.iter().any(|(id, _)| id == "b"));
    } else { panic!("Expected RankedList"); }
}

#[test]
fn test_composition_if() {
    let g = build_graph(vec![
        ("a", 5, 0, 0, "1"),
        ("b", 3, 1, 1, "2"),
    ]);
    let if_pred = IfPredicate {
        condition: Box::new(LargestPredicate),
        then_branch: Box::new(ColorPredicate { color: "1".into() }),
        else_branch: Box::new(ColorPredicate { color: "2".into() }),
    };
    if let PredicateResult::RankedList(list) = if_pred.evaluate(&g) {
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "a", "IF Largest THEN color=1 should return a");
    } else { panic!("Expected RankedList"); }
}
