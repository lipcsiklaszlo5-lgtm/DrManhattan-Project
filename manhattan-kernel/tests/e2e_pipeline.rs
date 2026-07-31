use manhattan_kernel::policy::{PolicyEngine, CostModel};
use manhattan_kernel::task::{Task, TaskType};
use manhattan_kernel::candidate::CandidateGenerator;
use manhattan_kernel::adapter::{DomainAdapter, Algorithm, ValidationError};
use manhattan_kernel::telemetry::Telemetry;

struct E2eTestAdapter;

impl DomainAdapter for E2eTestAdapter {
    fn build_structure(&self, _task: &Task) -> manhattan_kernel::structure::KernelStructureGraph {
        manhattan_kernel::structure::KernelStructureGraph::new()
    }

    fn validate(&self, _structure: &manhattan_kernel::structure::KernelStructureGraph, _candidate: &str) -> Result<(), ValidationError> {
        Ok(())
    }

    fn available_algorithms(&self) -> Vec<Algorithm> {
        Vec::new()
    }
}

#[test]
fn test_end_to_end_pipeline() {
    // 1. Inicializáció
    let cost_model = CostModel { llm_cost_per_call: 0.01 };
    let candidate_gen = CandidateGenerator::new(1);
    let mut engine = PolicyEngine::new(cost_model, candidate_gen);

    // 2. Feladat felépítése
    let mut task = Task::builder("fn main() {}")
        .task_type(TaskType::CodeGeneration)
        .build();

    // 3. Adapter és Telemetria példányosítás
    let adapter = E2eTestAdapter;
    let mut telemetry = Telemetry::new();

    // 4. A teljes E2E pipeline végrehajtása
    let result = engine.execute_task(&mut task, &adapter, &mut telemetry);
    
    // 5. Ellenőrzés
    assert!(result.is_ok(), "Az E2E pipeline-nak sikeresen le kell futnia.");
}
