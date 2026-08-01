use manhattan_kernel::arc_agi3_adapter::ArcAgi3Adapter;
use manhattan_kernel::meta_learner::MetaLearner;
use std::path::Path;

#[test]
fn test_arc_agi3_mock_tasks() {
    let path = Path::new("arc_agi3_tasks.json");
    let tasks = ArcAgi3Adapter::load_tasks(path).expect("Failed to load ARC-AGI-3 mock tasks");
    assert!(!tasks.is_empty(), "Should have at least one task");

    let mut learner = MetaLearner::new();
    let mut solved = 0;

    for task in &tasks {
        let instance = ArcAgi3Adapter::to_task_instance(task);
        if learner.learn_from_task(instance) {
            solved += 1;
        }
    }

    let accuracy = solved as f64 / tasks.len() as f64 * 100.0;
    println!("Solved {}/{} ARC-AGI-3 mock tasks ({:.1}%)", solved, tasks.len(), accuracy);
    
    assert!(solved > 0, "Should solve at least one ARC-AGI-3 mock task, got {}/{}", solved, tasks.len());
}
