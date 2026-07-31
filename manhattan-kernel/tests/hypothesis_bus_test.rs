use manhattan_kernel::hypothesis_bus::{HypothesisBus, BonsaiHypothesis};
use manhattan_kernel::policy::{PolicyEngine, CostModel};
use manhattan_kernel::candidate::CandidateGenerator;

#[test]
fn test_hypothesis_bus_basic() {
    let mut bus = HypothesisBus::new();
    assert!(bus.is_empty());
    bus.submit(BonsaiHypothesis {
        concept: "recolor".to_string(),
        confidence: 0.9,
        evidence: "test".to_string(),
    });
    assert!(!bus.is_empty());
    let hyps = bus.get_hypotheses();
    assert_eq!(hyps.len(), 1);
    assert_eq!(hyps[0].concept, "recolor");
    assert!(bus.is_empty());
}

#[test]
fn test_policy_engine_with_bus() {
    let mut bus = HypothesisBus::new();
    let cost_model = CostModel { llm_cost_per_call: 0.1 };
    let candidate_gen = CandidateGenerator::new(3);
    let _engine = PolicyEngine::new(cost_model, candidate_gen)
        .with_hypothesis_bus(&mut bus);
    // just check that we can build the engine with a bus
}
