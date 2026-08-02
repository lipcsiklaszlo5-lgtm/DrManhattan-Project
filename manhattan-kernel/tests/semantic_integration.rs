use manhattan_kernel::abstraction::program::{GeneralizedProgram, AbstractStep, Cardinality};
use manhattan_kernel::object_selector::ObjectSelector;
use manhattan_kernel::predicate::builtin::*;
use manhattan_kernel::sandbox::operators::Transformation;
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
fn test_generalized_program_uses_object_selector() {
    let step = AbstractStep {
        condition: Some(Box::new(LargestPredicate)),
        transformation: Transformation::NoOp,
        target_spec: None,
        cardinality: Cardinality::AtMostOne,
    };
    let prog = GeneralizedProgram {
        steps: vec![step],
        confidence: 1.0,
        num_train_pairs: 0,
    };
    let g = make_graph(vec![("a", 2, 0, 0, "1"), ("b", 5, 1, 1, "1")]);
    let matches = prog.matching_nodes(&g, 0);
    // Should select the largest node ("b")
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].node_id, "b");
}

#[test]
fn test_explanation_field_present() {
    let g = make_graph(vec![("a", 5, 0, 0, "1")]);
    let result = ObjectSelector::select_best_id(&LargestPredicate, &g);
    // Explanation is tested elsewhere; just ensure integration doesn't break
    assert!(result.is_some());
}

#[test]
fn test_and_predicate_with_selection() {
    let g = make_graph(vec![
        ("red_big", 10, 0, 0, "1"),
        ("red_small", 2, 1, 1, "1"),
        ("blue_big", 10, 2, 2, "2"),
    ]);
    let pred = manhattan_kernel::predicate::builtin::AndPredicate::new(
        Box::new(LargestPredicate),
        Box::new(ColorPredicate { color: "1".into() }),
    );
    let result = ObjectSelector::select(&pred, &g, &manhattan_kernel::object_selector::SelectionStrategy::Best, None);
    assert_eq!(result.selected[0].node_id, "red_big");
}
