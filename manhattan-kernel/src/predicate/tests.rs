#[cfg(test)]
mod predicate_tests {
    use crate::predicate::*;
    use crate::predicate::builtin::*;
    use crate::structure::KernelStructureGraph;

    fn make_graph(nodes: Vec<(String, &str, Vec<(&str, &str)>)>) -> KernelStructureGraph {
        let mut g = KernelStructureGraph::new();
        for (id, node_type, attrs) in nodes {
            let node = g.add_node(&id, node_type);
            for (k, v) in attrs {
                node.attributes.insert(k.to_string(), v.to_string());
            }
        }
        g
    }

    #[test]
    fn test_color_predicate() {
        let g = make_graph(vec![
            ("a".into(), "arc_object", vec![("color", "1"), ("area", "5")]),
            ("b".into(), "arc_object", vec![("color", "2"), ("area", "3")]),
        ]);
        let pred = ColorPredicate { color: "1".into() };
        if let PredicateResult::RankedList(list) = pred.evaluate(&g) {
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].0, "a");
        } else { panic!("Expected RankedList"); }
    }

    #[test]
    fn test_largest_smallest_predicates() {
        let g = make_graph(vec![
            ("a".into(), "arc_object", vec![("area", "5")]),
            ("b".into(), "arc_object", vec![("area", "8")]),
            ("c".into(), "arc_object", vec![("area", "3")]),
        ]);
        let largest = LargestPredicate.evaluate(&g);
        if let PredicateResult::RankedList(list) = largest {
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].0, "b");
        } else { panic!("Expected RankedList"); }

        let smallest = SmallestPredicate.evaluate(&g);
        if let PredicateResult::RankedList(list) = smallest {
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].0, "c");
        } else { panic!("Expected RankedList"); }
    }

    #[test]
    fn test_positional_predicates() {
        let g = make_graph(vec![
            ("a".into(), "arc_object", vec![("bbox_x", "0"), ("bbox_y", "0"), ("bbox_w", "2"), ("bbox_h", "2")]),
            ("b".into(), "arc_object", vec![("bbox_x", "3"), ("bbox_y", "1"), ("bbox_w", "1"), ("bbox_h", "1")]),
            ("c".into(), "arc_object", vec![("bbox_x", "1"), ("bbox_y", "3"), ("bbox_w", "1"), ("bbox_h", "1")]),
        ]);
        let leftmost = LeftmostPredicate.evaluate(&g);
        if let PredicateResult::RankedList(list) = leftmost {
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].0, "a");
        } else { panic!("Expected RankedList"); }

        let rightmost = RightmostPredicate.evaluate(&g);
        if let PredicateResult::RankedList(list) = rightmost {
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].0, "b");
        } else { panic!("Expected RankedList"); }
    }

    #[test]
    fn test_and_or_not() {
        let g = make_graph(vec![
            ("a".into(), "arc_object", vec![("color", "1"), ("area", "5")]),
            ("b".into(), "arc_object", vec![("color", "1"), ("area", "3")]),
            ("c".into(), "arc_object", vec![("color", "2"), ("area", "8")]),
        ]);
        let color1 = Box::new(ColorPredicate { color: "1".into() });
        let largest = Box::new(LargestPredicate);
        let and = AndPredicate { predicates: vec![color1.clone_box(), largest.clone_box()] };
        if let PredicateResult::RankedList(list) = and.evaluate(&g) {
            assert_eq!(list.len(), 0);
        } else { panic!("Expected RankedList (maybe empty)"); }

        let or = OrPredicate { predicates: vec![color1, largest] };
        if let PredicateResult::RankedList(list) = or.evaluate(&g) {
            assert!(list.iter().any(|(id, _)| id == "a" || id == "b" || id == "c"));
        } else { panic!("Expected RankedList"); }

        let not = NotPredicate { predicate: Box::new(ColorPredicate { color: "1".into() }) };
        if let PredicateResult::RankedList(list) = not.evaluate(&g) {
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].0, "c");
        } else { panic!("Expected RankedList"); }
    }

    #[test]
    fn test_connected_predicate() {
        let mut g = KernelStructureGraph::new();
        let _a = g.add_node("a", "arc_object");
        let _b = g.add_node("b", "arc_object");
        let _c = g.add_node("c", "arc_object");
        g.add_edge("a", "b", "touches");
        g.add_edge("b", "c", "touches");
        
        if let Some(node) = g.nodes.iter_mut().find(|n| n.id == "a") {
            node.attributes.insert("color".into(), "1".into());
        }
        let pred = ConnectedPredicate { reference: Box::new(ColorPredicate { color: "1".into() }) };
        if let PredicateResult::RankedList(list) = pred.evaluate(&g) {
            assert_eq!(list.len(), 2);
            assert!(list.iter().any(|(id, _)| id == "b"));
            assert!(list.iter().any(|(id, _)| id == "c"));
        } else { panic!("Expected RankedList"); }
    }

    #[test]
    fn test_inside_predicate() {
        let mut g = KernelStructureGraph::new();
        let _a = g.add_node("a", "arc_object");
        let _b = g.add_node("b", "arc_object");
        g.add_edge("a", "b", "contains");
        
        if let Some(node) = g.nodes.iter_mut().find(|n| n.id == "a") {
            node.attributes.insert("color".into(), "1".into());
        }
        let pred = InsidePredicate { reference: Box::new(ColorPredicate { color: "1".into() }) };
        if let PredicateResult::RankedList(list) = pred.evaluate(&g) {
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].0, "b");
        } else { panic!("Expected RankedList"); }
    }

    #[test]
    fn test_contains_predicate() {
        let mut g = KernelStructureGraph::new();
        let _a = g.add_node("a", "arc_object");
        let _b = g.add_node("b", "arc_object");
        g.add_edge("a", "b", "contains");
        
        if let Some(node) = g.nodes.iter_mut().find(|n| n.id == "b") {
            node.attributes.insert("color".into(), "1".into());
        }
        let pred = ContainsPredicate { reference: Box::new(ColorPredicate { color: "1".into() }) };
        if let PredicateResult::RankedList(list) = pred.evaluate(&g) {
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].0, "a");
        } else { panic!("Expected RankedList"); }
    }

    #[test]
    fn test_object_count_predicate() {
        let g = make_graph(vec![
            ("a".into(), "arc_object", vec![("color", "1")]),
            ("b".into(), "arc_object", vec![("color", "2")]),
        ]);
        let pred = ObjectCountPredicate { min: 2, max: 2 };
        assert_eq!(pred.evaluate(&g), PredicateResult::Bool(true));
    }

    #[test]
    fn test_neighbour_count_predicate() {
        let mut g = KernelStructureGraph::new();
        let _a = g.add_node("a", "arc_object");
        let _b = g.add_node("b", "arc_object");
        let _c = g.add_node("c", "arc_object");
        g.add_edge("a", "b", "touches");
        g.add_edge("a", "c", "touches");
        
        if let Some(node) = g.nodes.iter_mut().find(|n| n.id == "a") {
            node.attributes.insert("color".into(), "1".into());
        }
        let pred = NeighbourCountPredicate {
            reference: Box::new(ColorPredicate { color: "1".into() }),
            min: 2, max: 2,
        };
        if let PredicateResult::RankedList(list) = pred.evaluate(&g) {
            assert_eq!(list.len(), 1);
            assert_eq!(list[0].0, "a");
        } else { panic!("Expected RankedList"); }
    }
}
