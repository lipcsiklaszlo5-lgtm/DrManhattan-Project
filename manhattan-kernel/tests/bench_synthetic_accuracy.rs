use manhattan_kernel::sandbox::synthetic::SyntheticArcGenerator;
use manhattan_kernel::meta_learner::{MetaLearner, TaskInstance};

#[test]
fn bench_synthetic_accuracy() {
    let mut generator = SyntheticArcGenerator::new();
    let mut learner = MetaLearner::new();
    let num_tasks = 100;
    let mut solved = 0;

    for _ in 0..num_tasks {
        let (input, target, _ops) = generator.generate_task(5, 5, 1, 1); // egyszerű feladatok kezdetben
        let task = TaskInstance { grid: input, target };
        if learner.learn_from_task(task) {
            solved += 1;
        }
    }

    let accuracy = solved as f64 / num_tasks as f64 * 100.0;
    println!("Solved {}/{} tasks ({:.1}%)", solved, num_tasks, accuracy);
    // Kezdeti cél: legalább 20% egyszerű feladatokon
    assert!(accuracy >= 20.0, "Accuracy too low: {:.1}%", accuracy);
}
