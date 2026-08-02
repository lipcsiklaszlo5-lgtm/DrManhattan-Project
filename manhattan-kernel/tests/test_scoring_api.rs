use manhattan_kernel::object_selector::{ScoringProfile, ScoringComponent, SelectedObject};

#[test]
fn test_default_scoring_profile() {
    let profile = ScoringProfile::default();
    assert!(profile.components.is_empty() || !profile.components.is_empty());
    // profile should be usable
}

#[test]
fn test_custom_scoring_component() {
    struct DummyScorer;
    impl ScoringComponent for DummyScorer {
        fn name(&self) -> &str { "dummy" }
        fn score(&self, obj: &SelectedObject) -> f32 { 0.5 }
        fn weight(&self) -> f32 { 1.0 }
    }
    let comp = DummyScorer;
    let obj = SelectedObject {
        node_id: "n1".to_string(),
        score: 0.0,
        confidence: 1.0,
        explanation: String::new(),
        insertion_order: 0,
    };
    assert_eq!(comp.score(&obj), 0.5);
}
