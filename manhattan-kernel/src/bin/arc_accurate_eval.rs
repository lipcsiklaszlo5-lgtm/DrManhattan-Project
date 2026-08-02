use manhattan_kernel::adapter::arc::adapter::ArcAdapter;
use manhattan_kernel::adapter::arc::adapter::ArcGrid;
use manhattan_kernel::meta_learner::{MetaLearner, TaskInstance};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct ArcTask {
    train: Vec<ArcExample>,
    test: Vec<ArcExample>,
}

#[derive(Debug, Deserialize)]
struct ArcExample {
    input: Vec<Vec<u8>>,
    output: Vec<Vec<u8>>,
}

fn grid_from_2d(data: &[Vec<u8>]) -> ArcGrid {
    let height = data.len() as u8;
    let width = if height > 0 { data[0].len() as u8 } else { 0 };
    let pixels: Vec<u8> = data.iter().flatten().cloned().collect();
    ArcGrid::new(width, height, pixels)
}

fn main() {
    let tasks_dir = Path::new("ARC-AGI-master/data/training");
    if !tasks_dir.exists() {
        eprintln!("ARC dataset not found at {:?}", tasks_dir);
        return;
    }

    let mut total = 0;
    let mut solved = 0;
    let max_tasks = 5; // Csak 5 feladat a gyorsaság kedvéért
    let mut learner = MetaLearner::new();
    let start = Instant::now();

    if let Ok(entries) = fs::read_dir(tasks_dir) {
        for entry in entries.flatten() {
            if total >= max_tasks { break; }
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(task) = serde_json::from_str::<ArcTask>(&content) {
                        total += 1;

                        // Tanítás a train párokon (csak a tanulás, process_grid nélkül)
                        for example in &task.train {
                            let input_grid = grid_from_2d(&example.input);
                            let output_grid = grid_from_2d(&example.output);
                            let input_ksg = ArcAdapter::grid_to_ksg(&input_grid);
                            let target_ksg = ArcAdapter::grid_to_ksg(&output_grid);
                            // Közvetlenül a learn_generalized-et hívjuk, nem a learn_from_task-et
                            learner.program_synthesizer.learn_generalized(&input_ksg, &target_ksg, input_grid.width, input_grid.height);
                        }

                        // Értékelés a test páron
                        let mut task_solved = true;
                        for example in &task.test {
                            let input_grid = grid_from_2d(&example.input);
                            let target_grid = grid_from_2d(&example.output);

                            let input_ksg = ArcAdapter::grid_to_ksg(&input_grid);

                            // Csak generalizált programot használunk
                            if let Some(gen_prog) = learner.program_synthesizer.find_best_generalized() {
                                let result_ksg = gen_prog.apply(&input_ksg, target_grid.width, target_grid.height);
                                let result_grid = ArcAdapter::ksg_to_grid(&result_ksg, target_grid.width, target_grid.height, 0);
                                if result_grid.pixels != target_grid.pixels {
                                    task_solved = false;
                                }
                            } else {
                                task_solved = false;
                            }
                        }

                        if task_solved {
                            solved += 1;
                        }
                    }
                }
            }
        }
    }

    let elapsed = start.elapsed();
    let accuracy = if total > 0 { solved as f64 / total as f64 * 100.0 } else { 0.0 };
    println!("ARC-AGI-1 Fast Eval (first {} tasks):", total);
    println!("Solved: {}/{} ({:.1}%)", solved, total, accuracy);
    println!("Time: {:.2?}", elapsed);
    println!("Generalized programs: {}", learner.program_synthesizer.generalized_programs.len());
}
