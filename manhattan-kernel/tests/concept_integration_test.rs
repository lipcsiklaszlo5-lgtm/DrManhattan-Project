use manhattan_kernel::meta_learner::{MetaLearner, TaskInstance};
use manhattan_kernel::adapter::arc::adapter::{ArcGrid, ArcAdapter};
use manhattan_kernel::concept::Concept;

#[test]
fn test_meta_learner_discovers_concept() {
    let mut learner = MetaLearner::new();

    // Két hasonló "create" feladat, amelyek a Connected fogalmat váltják ki
    let task1 = TaskInstance {
        grid: ArcGrid::new(2, 2, vec![1, 0, 0, 0]),
        target: ArcGrid::new(2, 2, vec![1, 0, 0, 1]),
    };
    let task2 = TaskInstance {
        grid: ArcGrid::new(2, 2, vec![0, 0, 0, 0]),
        target: ArcGrid::new(2, 2, vec![0, 0, 0, 1]),
    };

    // Első feladat – még nem váltja ki a fogalmat, de tanul
    let success1 = learner.learn_from_task(task1);
    assert!(success1, "First task should be solved");

    // Második feladat – most már a ConceptLearner kétszer látja a "create_1" mintát
    let success2 = learner.learn_from_task(task2);
    assert!(success2, "Second task should be solved");

    // Ellenőrizzük, hogy a ConceptRegistry tartalmazza a Connected fogalmat
    let dummy_ksg = ArcAdapter::grid_to_ksg(&ArcGrid::new(1, 1, vec![0]));
    let scanned = learner.concept_registry.scan(&dummy_ksg);
    assert!(
        scanned.contains(&Concept::Connected),
        "ConceptRegistry should contain Connected after two similar create diffs, got: {:?}",
        scanned
    );

    // Ellenőrizzük, hogy a fogalom a ProgramSynthesizer-ben is rögzült
    let transforms = learner.program_synthesizer.transforms_for_concept(&Concept::Connected);
    assert!(!transforms.is_empty(), "Connected concept should map to at least one transformation");
}
