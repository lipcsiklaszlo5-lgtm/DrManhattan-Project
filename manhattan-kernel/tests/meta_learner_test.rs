use manhattan_kernel::meta_learner::{MetaLearner, TaskInstance};
use manhattan_kernel::adapter::arc::adapter::ArcGrid;

#[test]
fn test_meta_learner_one_shot_simple() {
    let mut learner = MetaLearner::new();
    let input = ArcGrid::new(3, 3, vec![0,0,0, 0,0,0, 0,0,0]);
    let target = ArcGrid::new(3, 3, vec![0,0,0, 0,1,0, 0,0,0]);
    let task = TaskInstance { grid: input, target };
    let success = learner.learn_from_task(task);
    assert!(success, "Meta-learner should solve simple add-pixel task");
}

#[test]
fn test_meta_learner_adapts_on_failure() {
    let mut learner = MetaLearner::new();
    let input = ArcGrid::new(3, 3, vec![1,0,0, 0,0,0, 0,0,0]);
    let target = ArcGrid::new(3, 3, vec![0,0,0, 0,2,0, 0,0,0]);
    let task = TaskInstance { grid: input, target };
    let _ = learner.learn_from_task(task);
    // just ensure no panic
}
