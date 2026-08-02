use manhattan_kernel::meta_learner::{MetaLearner, TaskInstance};
use manhattan_kernel::adapter::arc::adapter::{ArcGrid, ArcAdapter};
use manhattan_kernel::structure::KernelStructureGraph;
use manhattan_kernel::abstraction::program::ProgramSynthesizer;
use manhattan_kernel::abstraction::hypothesis::HypothesisManager;
use std::fs;
use std::path::Path;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
struct DiagnosisReport {
    task_file: String,
    train_pairs: usize,
    representations: Vec<RepresentationInfo>,
    hypotheses: Vec<HypothesisInfo>,
    generalized_programs: Vec<ProgramInfo>,
    test_result: TestResult,
    failure_analysis: String,
}

#[derive(Debug, Serialize)]
struct RepresentationInfo {
    name: String,
    node_count: usize,
    edge_count: usize,
}

#[derive(Debug, Serialize)]
struct HypothesisInfo {
    representation: String,
    program_exists: bool,
    cost: f64,
    success_rate: f64,
}

#[derive(Debug, Serialize)]
struct ProgramInfo {
    step_count: usize,
    conditions: Vec<String>,
    transformations: Vec<String>,
    confidence: f32,
}

#[derive(Debug, Serialize)]
struct TestResult {
    solved: bool,
    predicted_grid: Option<Vec<Vec<u8>>>,
    expected_grid: Vec<Vec<u8>>,
    predicted_ksg_node_count: usize,
    expected_ksg_node_count: usize,
}

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
    if args.len() < 2 {
        eprintln!("Usage: arc_diagnose <task_json_file>");
        std::process::exit(1);
    }
    let task_path = Path::new(&args[1]);
    let task = match load_arc_task(task_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to load task: {}", e);
            std::process::exit(1);
        }
    };

    let mut learner = MetaLearner::new();
    let train_pairs = task["train"].as_array().map(|a| a.len()).unwrap_or(0);

    // Train on all examples
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
            let instance = TaskInstance { grid: input, target: output };
            learner.learn_from_task(instance);
        }
    }

    // Collect diagnostic info from internal state (public fields)
    let representations: Vec<RepresentationInfo> = learner.hypothesis_manager.hypotheses.iter()
        .map(|h| {
            // We don't have direct access to the graph here, but we can get the representation name
            RepresentationInfo {
                name: h.representation_name.clone(),
                node_count: 0, // not directly available without graph reference
                edge_count: 0,
            }
        })
        .collect();

    let hypotheses: Vec<HypothesisInfo> = learner.hypothesis_manager.hypotheses.iter()
        .map(|h| HypothesisInfo {
            representation: h.representation_name.clone(),
            program_exists: h.program.is_some(),
            cost: h.cost,
            success_rate: h.success_rate(),
        })
        .collect();

    let generalized_programs: Vec<ProgramInfo> = learner.program_synthesizer.generalized_programs.iter()
        .map(|gp| ProgramInfo {
            step_count: gp.steps.len(),
            conditions: gp.steps.iter().map(|s| s.condition.as_ref().map(|c| c.name().to_string()).unwrap_or_default()).collect(),
            transformations: gp.steps.iter().map(|s| format!("{:?}", s.transformation)).collect(),
            confidence: gp.confidence,
        })
        .collect();

    // Test on first test example
    let test_result = if let Some(test_examples) = task["test"].as_array() {
        let example = &test_examples[0];
        let test_input = match grid_from_json(&example["input"]) {
            Ok(g) => g,
            Err(_) => {
                eprintln!("Invalid test input");
                std::process::exit(1);
            }
        };
        let expected_output = match grid_from_json(&example["output"]) {
            Ok(g) => g,
            Err(_) => {
                eprintln!("Invalid test output");
                std::process::exit(1);
            }
        };

        let prediction = learner.predict(&test_input);
        let solved = prediction.as_ref().map_or(false, |p| p.pixels == expected_output.pixels);

        let predicted_ksg = prediction.as_ref().map(|p| ArcAdapter::grid_to_ksg(p));
        let expected_ksg = ArcAdapter::grid_to_ksg(&expected_output);

        TestResult {
            solved,
            predicted_grid: prediction.as_ref().map(grid_to_2d),
            expected_grid: grid_to_2d(&expected_output),
            predicted_ksg_node_count: predicted_ksg.map(|k| k.nodes.len()).unwrap_or(0),
            expected_ksg_node_count: expected_ksg.nodes.len(),
        }
    } else {
        TestResult {
            solved: false,
            predicted_grid: None,
            expected_grid: Vec::new(),
            predicted_ksg_node_count: 0,
            expected_ksg_node_count: 0,
        }
    };

    // Failure analysis
    let failure_analysis = if test_result.solved {
        "Success".to_string()
    } else {
        let mut reasons = Vec::new();
        if learner.program_synthesizer.generalized_programs.is_empty() {
            reasons.push("No generalized programs learned".to_string());
        }
        if hypotheses.is_empty() {
            reasons.push("No hypotheses generated".to_string());
        }
        if test_result.predicted_ksg_node_count != test_result.expected_ksg_node_count {
            reasons.push(format!(
                "Node count mismatch: predicted {}, expected {}",
                test_result.predicted_ksg_node_count, test_result.expected_ksg_node_count
            ));
        }
        if reasons.is_empty() {
            "Unknown reason – program exists but output does not match".to_string()
        } else {
            reasons.join("; ")
        }
    };

    let report = DiagnosisReport {
        task_file: task_path.file_name().unwrap().to_string_lossy().to_string(),
        train_pairs,
        representations,
        hypotheses,
        generalized_programs,
        test_result,
        failure_analysis,
    };

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
