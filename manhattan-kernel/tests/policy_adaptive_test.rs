use manhattan_kernel::policy::{PolicyEngine, CostModel, StrategyStats};
use manhattan_kernel::candidate::CandidateGenerator;
use manhattan_kernel::task::{Task, TaskBuilder};
use manhattan_kernel::adapter::DomainAdapter;
use manhattan_kernel::structure::KernelStructureGraph;
use manhattan_kernel::adapter::Algorithm;

struct DummyAdapter;
impl DomainAdapter for DummyAdapter {
    fn build_structure(&self, _task: &Task) -> KernelStructureGraph {
        // Visszaadunk egy nem üres gráfot, hogy ne a "success" útvonalat válassza
        let mut g = KernelStructureGraph::new();
        g.add_node("test_node", "compiler_error");
        g
    }
    fn graph_to_code(&self, _graph: &KernelStructureGraph, _orig: &str) -> String { "ok".to_string() }
    fn validate(&self, _graph: &KernelStructureGraph, _code: &str) -> Result<(), manhattan_kernel::adapter::ValidationError> {
        Ok(())
    }
    fn available_algorithms(&self) -> Vec<Algorithm> {
        vec![]
    }
}

#[test]
fn test_adaptive_decide_prefers_high_success_strategy() {
    let cost_model = CostModel { llm_cost_per_call: 0.1 };
    let candidate_gen = CandidateGenerator::new(3);
    let mut engine = PolicyEngine::new(cost_model, candidate_gen);
    // Kézzel beállítjuk a statisztikát
    engine.strategy_stats.insert("algorithm".to_string(), StrategyStats { successes: 10, failures: 1 });
    engine.strategy_stats.insert("llm".to_string(), StrategyStats { successes: 1, failures: 10 });
    
    let adapter = DummyAdapter;
    let mut task = TaskBuilder::new("test").build();
    task.context.structure = Some(adapter.build_structure(&task));
    
    // Olyan task, aminek van structure-je és nincs grid-je -> elérhető: cache, algorithm, llm
    // Mivel a cache üres, az algorithm és llm közül választ
    let decision = engine.decide(&task, &adapter);
    assert_eq!(decision, "algorithm", "Should pick algorithm due to higher success rate");
}
