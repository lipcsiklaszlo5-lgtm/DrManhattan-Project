use manhattan_kernel::adapter::arc::adapter::ArcAdapter;
use manhattan_kernel::adapter::arc::adapter::ArcGrid;
use manhattan_kernel::meta_learner::{MetaLearner, TaskInstance};
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

    let mut total = 0;
    let mut solved = 0;
    let max_tasks = 5; // Csak 5 feladatot nézünk a gyorsaság kedvéért
    let mut learner = MetaLearner::new();

    if let Ok(entries) = fs::read_dir(tasks_dir) {
        for entry in entries.flatten() {
            if total >= max_tasks { break; }
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(task) = serde_json::from_str::<ArcTask>(&content) {
                        total += 1;
                        // Csak tanítás, tesztelés nélkül
                        for example in &task.train {
                            let input_grid = grid_from_2d(&example.input);
                            let output_grid = grid_from_2d(&example.output);
                            let instance = TaskInstance { grid: input_grid, target: output_grid };
                            learner.learn_from_task(instance);
                        }
                    }
                }
            }
        }
    }

    println!("Processed {} tasks. Programs: {}, Generalized: {}",
             total,
             learner.program_synthesizer.programs.len(),
             learner.program_synthesizer.generalized_programs.len());
}
