use manhattan_kernel::meta_learner::{MetaLearner, TaskInstance};
use manhattan_kernel::adapter::arc::adapter::ArcGrid;
use std::fs;
use std::path::Path;
use serde::Serialize;
use serde_json::Value;

// ================================================================
// Strongly typed regression state
// ================================================================

#[derive(Debug, Clone, Serialize)]
struct TaskEntry {
    task_file: String,
    test_index: usize,
    solved: bool,
    predicted: Option<Vec<Vec<u8>>>,
}

#[derive(Debug, Clone, Serialize)]
struct RegressionReport {
    solved_train: u64,
    total_train: u64,
    solved_test: u64,
    total_test: u64,
    tasks: Vec<TaskEntry>,
    mode: String,
}

impl RegressionReport {
    fn new(mode: &str) -> Self {
        Self {
            solved_train: 0,
            total_train: 0,
            solved_test: 0,
            total_test: 0,
            tasks: Vec::new(),
            mode: mode.to_string(),
        }
    }
}

// ================================================================
// Helpers
// ================================================================

fn load_arc_task(path: &Path) -> Result<Value, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
    let task: Value = serde_json::from_str(&content).map_err(|e| format!("JSON error: {}", e))?;
    if !task["train"].is_array() || !task["test"].is_array() {
        return Err("Invalid ARC task format".to_string());
    }
    Ok(task)
}

fn grid_from_json(grid: &Value) -> Result<ArcGrid, String> {
    let pixels: Vec<Vec<u8>> = grid.as_array()
        .ok_or("Grid not array")?
        .iter()
        .map(|row| row.as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect())
        .collect();
    let height = pixels.len();
    let width = pixels.first().map_or(0, |r| r.len());
    Ok(ArcGrid {
        width: width as u8,
        height: height as u8,
        pixels: pixels.concat(),
    })
}

fn grid_to_2d(grid: &ArcGrid) -> Vec<Vec<u8>> {
    let mut rows = Vec::new();
    for y in 0..grid.height {
        let start = (y as usize) * (grid.width as usize);
        let row: Vec<u8> = grid.pixels[start..start + grid.width as usize].to_vec();
        rows.push(row);
    }
    rows
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut mode = "smoke".to_string();
    let mut data_dir_opt: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--mode" => {
                i += 1;
                mode = args.get(i).cloned().unwrap_or_else(|| "smoke".to_string());
            }
            other => {
                if !other.starts_with("--") {
                    data_dir_opt = Some(other.to_string());
                }
            }
        }
        i += 1;
    }

    let data_dir = match data_dir_opt {
        Some(d) => d,
        None => {
            eprintln!("Usage: arc_regression_runner [--mode smoke|small|full] <arc_data_dir>");
            std::process::exit(1);
        }
    };
    let data_path = Path::new(&data_dir);

    // Collect all task files
    let mut task_files: Vec<_> = Vec::new();
    if let Ok(entries) = fs::read_dir(data_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                task_files.push(path);
            }
        }
    }
    task_files.sort();

    // Select subset based on mode
    let selected_tasks = match mode.as_str() {
        "smoke" => task_files.into_iter().take(1).collect::<Vec<_>>(),
        "small" => task_files.into_iter().take(5).collect::<Vec<_>>(),
        _ => task_files,
    };

    let mut report = RegressionReport::new(&mode);

    for task_path in selected_tasks {
        let task = match load_arc_task(&task_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Skipping {}: {}", task_path.display(), e);
                continue;
            }
        };

        let mut learner = MetaLearner::new();

        // Train on all training examples
        if let Some(train_examples) = task["train"].as_array() {
            for example in train_examples {
                let input = match grid_from_json(&example["input"]) {
                    Ok(g) => g,
                    Err(_) => continue,
                };
                let output = match grid_from_json(&example["output"]) {
                    Ok(g) => g,
                    Err(_) => continue,
                };
                let instance = TaskInstance { grid: input.clone(), target: output.clone() };
                learner.learn_from_task(instance);

                // Check train accuracy
                if let Some(pred) = learner.predict(&input) {
                    if pred.pixels == output.pixels {
                        report.solved_train += 1;
                    }
                }
                report.total_train += 1;
            }
        }

        learner.finalize();

        // Test examples
        if let Some(test_examples) = task["test"].as_array() {
            for (i, example) in test_examples.iter().enumerate() {
                let test_input = match grid_from_json(&example["input"]) {
                    Ok(g) => g,
                    Err(_) => continue,
                };
                let expected_output = match grid_from_json(&example["output"]) {
                    Ok(g) => g,
                    Err(_) => continue,
                };

                let prediction = learner.predict(&test_input);
                let solved = prediction.as_ref().map_or(false, |p| p.pixels == expected_output.pixels);

                if solved {
                    report.solved_test += 1;
                }
                report.total_test += 1;

                let entry = TaskEntry {
                    task_file: task_path.file_name().unwrap().to_string_lossy().to_string(),
                    test_index: i,
                    solved,
                    predicted: prediction.as_ref().map(grid_to_2d),
                };
                report.tasks.push(entry);
            }
        }
    }

    // Serialize once at the end
    let json_output = serde_json::to_string_pretty(&report).expect("Serialization failed");
    println!("{}", json_output);
}
