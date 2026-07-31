use manhattan_kernel::meta_learner::{MetaLearner, TaskInstance};
use manhattan_kernel::adapter::arc::adapter::ArcGrid;

/// End-to-end test: a single colored pixel is translated.
/// The MetaLearner should solve it via one-shot AgentLoop.
#[test]
fn test_e2e_simple_translate() {
    let mut learner = MetaLearner::new();

    // 3x3 grid, one blue pixel at (0,0) -> (2,2)
    let input = ArcGrid::new(3, 3, vec![
        1, 0, 0,
        0, 0, 0,
        0, 0, 0,
    ]);
    let target = ArcGrid::new(3, 3, vec![
        0, 0, 0,
        0, 0, 0,
        0, 0, 1,
    ]);

    let task = TaskInstance { grid: input, target };
    let success = learner.learn_from_task(task);
    assert!(success, "MetaLearner must solve simple translate task");

    // Verify that at least one program was learned
    assert!(!learner.program_synthesizer.programs.is_empty(),
            "At least one program must have been learned");
    
    // Verify the learned program is a Translate (by checking steps)
    let learned = learner.program_synthesizer.programs.last().unwrap();
    assert!(!learned.steps.is_empty(), "Program must have steps");
    assert!(learned.steps.iter().any(|s| matches!(s, manhattan_kernel::sandbox::operators::Transformation::Translate { .. })),
            "Learned program must contain Translate");
}
