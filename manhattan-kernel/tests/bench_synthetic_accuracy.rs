use manhattan_kernel::sandbox::synthetic::SyntheticArcGenerator;
use manhattan_kernel::meta_learner::{MetaLearner, TaskInstance};
use manhattan_kernel::adapter::arc::adapter::ArcAdapter;

#[test]
fn bench_synthetic_accuracy() {
    let mut generator = SyntheticArcGenerator::new();
    let mut learner = MetaLearner::new();
    let num_tasks = 100;
    let mut solved = 0;
    let mut false_positives = 0;

    for _ in 0..num_tasks {
        let (input, target, _ops) = generator.generate_task(5, 5, 1, 1);
        let task = TaskInstance { grid: input.clone(), target: target.clone() };
        
        let result = learner.learn_from_task(task);
        
        let input_ksg = ArcAdapter::grid_to_ksg(&input);
        let target_ksg = ArcAdapter::grid_to_ksg(&target);
        let best_prog = learner.program_synthesizer.find_best_program(&input_ksg, &target_ksg);
        
        if let Some(prog) = best_prog {
            let result_ksg = prog.apply(&input_ksg);
            let result_grid = ArcAdapter::ksg_to_grid(&result_ksg, target.width, target.height, 0);
            if result_grid.pixels == target.pixels {
                solved += 1;
            } else if result {
                false_positives += 1;
            }
        } else if result {
            false_positives += 1;
        }
    }

    let accuracy = solved as f64 / num_tasks as f64 * 100.0;
    println!("Solved {}/{} tasks ({:.1}%)", solved, num_tasks, accuracy);
    println!("False positives: {}", false_positives);
    // Csak tájékoztató jelleggel, nem buktatjuk a tesztet
    if false_positives > 0 {
        println!("WARNING: {} false positives detected", false_positives);
    }
}
