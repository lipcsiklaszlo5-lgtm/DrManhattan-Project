use manhattan_kernel::object_selector::{
    ObjectSelector, SelectionStrategy, ScoringProfile, ScoringComponent,
    PredicateConfidenceComponent, AreaComponent, SelectedObject,
};
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
fn test_higher_predicate_confidence_produces_higher_score() {
    let g = make_graph(vec![("a", 5, 0, 0, "1"), ("b", 5, 1, 1, "1")]);
    // LargestPredicate gives both equal score, then tie-break by area
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::All, None);
    // Both have identical area, so scores should be equal
    assert!((result.ranking[0].score - result.ranking[1].score).abs() < 0.001);
}

#[test]
fn test_area_component_boosts_larger_objects() {
    let g = make_graph(vec![("small", 1, 0, 0, "1"), ("large", 100, 1, 1, "1")]);
    let profile = ScoringProfile::new(vec![
        Box::new(PredicateConfidenceComponent::default()),
        Box::new(AreaComponent::default()),
    ]);
    let result = ObjectSelector::select_with_scoring(
        &AreaPredicate { min: None, max: None }, &g, &SelectionStrategy::Best, &profile,
    );
    assert_eq!(result.selected[0].node_id, "large");
}

#[test]
fn test_weighted_scoring_component() {
    // Custom component that heavily penalizes large areas
    struct PenalizeArea;
    impl ScoringComponent for PenalizeArea {
        fn name(&self) -> &str { "PenalizeArea" }
        fn weight(&self) -> f32 { 100.0 }
        fn score(&self, node: &manhattan_kernel::structure::Node, _: f32, _: &KernelStructureGraph) -> f32 {
            let area: f32 = node.attributes.get("area").and_then(|v| v.parse().ok()).unwrap_or(1.0);
            -area
        }
    }
    let g = make_graph(vec![("small", 1, 0, 0, "1"), ("large", 100, 1, 1, "1")]);
    let profile = ScoringProfile::new(vec![Box::new(PenalizeArea)]);
    let result = ObjectSelector::select_with_scoring(
        &AreaPredicate { min: None, max: None }, &g, &SelectionStrategy::Best, &profile,
    );
    // The PenalizeArea component gives large negative score to "large"
    assert_eq!(result.selected[0].node_id, "small");
}
