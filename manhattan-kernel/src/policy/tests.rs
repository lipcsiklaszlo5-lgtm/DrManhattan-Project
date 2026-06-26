#[cfg(test)]
mod tests {
    use crate::policy::{CostModel, PolicyEngine};
    use crate::candidate::CandidateGenerator;
    use crate::task::{Task, TaskType};
    use crate::structure::KernelStructureGraph;

    fn make_task(with_structure: bool) -> Task {
        let builder = Task::builder("test intent").task_type(TaskType::Unknown);
        if with_structure {
            let mut g = KernelStructureGraph::new();
            g.add_node("n1", "error");
            builder.structure(g).build()
        } else {
            builder.build()
        }
    }

    #[test]
    fn test_decide_algorithm() {
        let engine = PolicyEngine::new(CostModel { llm_cost_per_call: 0.01 }, CandidateGenerator::new(1));
        let task = make_task(false);
        assert_eq!(engine.decide(&task, true, false), "algorithm");
    }

    #[test]
    fn test_decide_cache() {
        let engine = PolicyEngine::new(CostModel { llm_cost_per_call: 0.01 }, CandidateGenerator::new(1));
        let task = make_task(false);
        assert_eq!(engine.decide(&task, false, true), "cache");
    }

    #[test]
    fn test_decide_local_search() {
        let engine = PolicyEngine::new(CostModel { llm_cost_per_call: 0.01 }, CandidateGenerator::new(1));
        let task = make_task(true);
        assert_eq!(engine.decide(&task, false, false), "local_search");
    }

    #[test]
    fn test_decide_llm_when_cheap() {
        let engine = PolicyEngine::new(CostModel { llm_cost_per_call: 0.01 }, CandidateGenerator::new(1));
        let task = make_task(false);
        assert_eq!(engine.decide(&task, false, false), "llm");
    }
}
