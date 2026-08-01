use manhattan_kernel::meta_learner::{MetaLearner, TaskInstance};
use manhattan_kernel::adapter::arc::adapter::ArcAdapter;
use manhattan_kernel::adapter::arc::adapter::ArcGrid;
use serde::Deserialize;
use std::fs;
use std::path::Path;

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

    let mut total_tasks = 0;
    let mut solved_tasks = 0;
    let mut learner = MetaLearner::new();

    if let Ok(entries) = fs::read_dir(tasks_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(task) = serde_json::from_str::<ArcTask>(&content) {
                        total_tasks += 1;

                        for example in &task.train {
                            let input_grid = grid_from_2d(&example.input);
                            let output_grid = grid_from_2d(&example.output);
                            let task_instance = TaskInstance {
                                grid: input_grid,
                                target: output_grid,
                            };
                            learner.learn_from_task(task_instance);
                        }

                        let mut task_solved = true;
                        for example in &task.test {
                            let input_grid = grid_from_2d(&example.input);
                            let target_grid = grid_from_2d(&example.output);
                            
                            let input_ksg = ArcAdapter::grid_to_ksg(&input_grid);
                            let target_ksg = ArcAdapter::grid_to_ksg(&target_grid);
                            
                            if let Some(best_prog) = learner.program_synthesizer.find_best_program(&input_ksg, &target_ksg) {
                                let result_ksg = best_prog.apply(&input_ksg);
                                let result_grid = ArcAdapter::ksg_to_grid(&result_ksg, target_grid.width, target_grid.height, 0);
                                if result_grid.pixels != target_grid.pixels {
                                    task_solved = false;
                                }
                            } else {
                                task_solved = false;
                            }
                        }

                        if task_solved {
                            solved_tasks += 1;
                        }
                    }
                }
            }
        }
    }

    let accuracy = if total_tasks > 0 {
        solved_tasks as f64 / total_tasks as f64 * 100.0
    } else {
        0.0
    };

    println!("ARC-AGI-1 Evaluation Results:");
    println!("Total tasks: {}", total_tasks);
    println!("Solved tasks: {}", solved_tasks);
    println!("Accuracy: {:.1}%", accuracy);
    println!("Programs learned: {}", learner.program_synthesizer.programs.len());
    println!("Concepts discovered: {:?}", learner.concept_learner.discovered);
}
