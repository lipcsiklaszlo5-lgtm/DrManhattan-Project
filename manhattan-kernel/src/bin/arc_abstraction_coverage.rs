use manhattan_kernel::meta_learner::{MetaLearner, TaskInstance};
use manhattan_kernel::adapter::arc::adapter::{ArcGrid, ArcAdapter};
use manhattan_kernel::abstraction::program::{GeneralizedProgram};
use manhattan_kernel::structure::KernelStructureGraph;
use std::fs;
use std::path::Path;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
struct CoverageReport {
    task_file: String,
    train_pairs: usize,
    programs: Vec<ProgramCoverage>,
    best_coverage: f64,
    failure_details: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProgramCoverage {
    program_index: usize,
    steps: usize,
    conditions: Vec<String>,
    transformations: Vec<String>,
    pairs_covered: usize,
    total_pairs: usize,
    coverage: f64,
}

fn load_arc_task(path: &Path) -> Result<Value, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
    let task: Value = serde_json::from_str(&content).map_err(|e| format!("JSON error: {}", e))?;
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
    Ok(ArcGrid { width: width as u8, height: height as u8, pixels: pixels.concat() })
}

fn apply_program_to_grid(program: &GeneralizedProgram, input: &ArcGrid) -> Option<ArcGrid> {
    let ksg = ArcAdapter::grid_to_ksg(input);
    let result_ksg = program.apply(&ksg, input.width, input.height);
    Some(ArcAdapter::ksg_to_grid(&result_ksg, input.width, input.height, 0))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: arc_abstraction_coverage <task_json>");
        std::process::exit(1);
    }
    let task_path = Path::new(&args[1]);
    let task = match load_arc_task(task_path) {
        Ok(t) => t,
        Err(e) => { eprintln!("Error: {}", e); std::process::exit(1); }
    };

    let mut learner = MetaLearner::new();
    let train_examples = task["train"].as_array().unwrap();
    let total_pairs = train_examples.len();

    // Tanítás -- kozben osszegyujtjuk a train par grid-eket a finalize()
    // szamara is (klonozva, mert az eredeti input/output a TaskInstance-be
    // mozog).
    let mut train_pairs: Vec<(ArcGrid, ArcGrid)> = Vec::with_capacity(total_pairs);
    for example in train_examples {
        let input = grid_from_json(&example["input"]).unwrap();
        let output = grid_from_json(&example["output"]).unwrap();
        train_pairs.push((input.clone(), output.clone()));
        learner.learn_from_task(TaskInstance { grid: input, target: output });
    }
    learner.finalize(&train_pairs);

    // Értékelés: minden generalizált programot tesztelünk az összes train páron
    let programs = &learner.program_synthesizer.generalized_programs;
    let mut coverages = Vec::new();
    let mut best_coverage = 0.0f64;
    let mut failure_details = Vec::new();

    for (idx, program) in programs.iter().enumerate() {
        let mut pairs_covered = 0;
        for example in train_examples {
            let input = grid_from_json(&example["input"]).unwrap();
            let expected = grid_from_json(&example["output"]).unwrap();
            if let Some(predicted) = apply_program_to_grid(program, &input) {
                if predicted.pixels == expected.pixels {
                    pairs_covered += 1;
                }
            }
        }
        let coverage = pairs_covered as f64 / total_pairs as f64;
        if coverage > best_coverage {
            best_coverage = coverage;
        }

        // Rögzítsük a lépések részleteit
        let conditions: Vec<String> = program.steps.iter()
            .map(|s| s.condition.as_ref().map(|c| c.name().to_string()).unwrap_or_default())
            .collect();
        let transformations: Vec<String> = program.steps.iter()
            .map(|s| format!("{:?}", s.transformation))
            .collect();

        // Ha nem teljes a fedés, gyűjtsünk hibát
        if coverage < 1.0 {
            for (i, example) in train_examples.iter().enumerate() {
                let input = grid_from_json(&example["input"]).unwrap();
                let expected = grid_from_json(&example["output"]).unwrap();
                if let Some(predicted) = apply_program_to_grid(program, &input) {
                    if predicted.pixels != expected.pixels {
                        let input_ksg = ArcAdapter::grid_to_ksg(&input);
                        let expected_ksg = ArcAdapter::grid_to_ksg(&expected);
                        let pred_ksg = ArcAdapter::grid_to_ksg(&predicted);
                        failure_details.push(format!(
                            "Program {} pair {}: node count mismatch (pred {} vs exp {}) | conditions: {:?}",
                            idx, i, pred_ksg.nodes.len(), expected_ksg.nodes.len(), conditions
                        ));
                    }
                }
            }
        }

        coverages.push(ProgramCoverage {
            program_index: idx,
            steps: program.steps.len(),
            conditions,
            transformations,
            pairs_covered,
            total_pairs,
            coverage,
        });
    }

    let report = CoverageReport {
        task_file: task_path.file_name().unwrap().to_string_lossy().to_string(),
        train_pairs: total_pairs,
        programs: coverages,
        best_coverage,
        failure_details,
    };

    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
