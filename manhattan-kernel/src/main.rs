use manhattan_kernel::task::{Task, TaskType};
use manhattan_kernel::adapter::compiler::CompilerAdapter;
use manhattan_kernel::policy::{CostModel, PolicyEngine};
use manhattan_kernel::candidate::CandidateGenerator;
use manhattan_kernel::telemetry::Telemetry;
use std::env;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <rust_code>", args[0]);
        std::process::exit(1);
    }
    let code = args[1].clone();

    let cost_model = CostModel { llm_cost_per_call: 0.01 };
    let candidate_gen = CandidateGenerator::new(1);
    let mut engine = PolicyEngine::new(cost_model, candidate_gen);
    let adapter = CompilerAdapter;
    let mut telemetry = Telemetry::new();
    let mut task = Task::builder(code).task_type(TaskType::CodeGeneration).build();

    match engine.execute_task(&mut task, &adapter, &mut telemetry) {
        Ok(solution) => println!("{}", solution),
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}
