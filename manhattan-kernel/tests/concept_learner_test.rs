use manhattan_kernel::concept_learner::ConceptLearner;
use manhattan_kernel::concept::ConceptRegistry;
use manhattan_kernel::adapter::arc::adapter::ArcAdapter;
use manhattan_kernel::adapter::arc::adapter::ArcGrid;

#[test]
fn test_concept_learner_discovers_create() {
    let mut learner = ConceptLearner::new();
    let mut registry = ConceptRegistry::default();
    // First occurrence
    let before_grid = ArcGrid::new(2, 2, vec![1, 0, 0, 0]);
    let after_grid = ArcGrid::new(2, 2, vec![1, 0, 0, 1]);
    let before_ksg = ArcAdapter::grid_to_ksg(&before_grid);
    let after_ksg = ArcAdapter::grid_to_ksg(&after_grid);
    let new_concepts = learner.learn_from_diff(&before_ksg, &after_ksg, &mut registry);
    assert!(new_concepts.is_empty(), "First occurrence should not create concept");

    // Second occurrence
    let before_grid2 = ArcGrid::new(2, 2, vec![0, 0, 0, 0]);
    let after_grid2 = ArcGrid::new(2, 2, vec![0, 0, 0, 1]);
    let before_ksg2 = ArcAdapter::grid_to_ksg(&before_grid2);
    let after_ksg2 = ArcAdapter::grid_to_ksg(&after_grid2);
    let new_concepts2 = learner.learn_from_diff(&before_ksg2, &after_ksg2, &mut registry);
    assert!(!new_concepts2.is_empty(), "Should discover concept after two similar diffs");

    // Ellenőrizzük, hogy a detektor működik
    let scanned = registry.scan(&after_ksg2);
    assert!(scanned.contains(&manhattan_kernel::concept::Concept::Connected),
            "Registry should detect Connected via dynamic detector");
}
