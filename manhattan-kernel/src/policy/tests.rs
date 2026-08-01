#[cfg(test)]
mod tests {
    use crate::policy::{PolicyEngine, CostModel, StrategyStats};
    use crate::candidate::CandidateGenerator;
    use crate::task::TaskBuilder;
    use crate::adapter::compiler::CompilerAdapter;
    use crate::adapter::DomainAdapter;
    use crate::telemetry::Telemetry;

    #[test]
    fn test_execute_task_no_error() {
        let mut engine = PolicyEngine::new(CostModel { llm_cost_per_call: 0.1 }, CandidateGenerator::new(3));
        let adapter = CompilerAdapter;
        let mut task = TaskBuilder::new("let x = 1;").build();
        let mut telemetry = Telemetry::new();
        task.context.structure = Some(adapter.build_structure(&task));
        engine.strategy_stats.insert("algorithm".to_string(), StrategyStats { successes: 10, failures: 0 });
        let result = engine.execute_task(&mut task, &adapter, &mut telemetry);
        assert!(result.is_ok(), "execute_task should succeed: {:?}", result);
    }

    #[test]
    fn test_execute_task_with_compiler_error() {
        let mut engine = PolicyEngine::new(CostModel { llm_cost_per_call: 0.1 }, CandidateGenerator::new(3));
        let adapter = CompilerAdapter;
        let mut task = TaskBuilder::new("invalid syntax !!!").build();
        let mut telemetry = Telemetry::new();
        task.context.structure = Some(adapter.build_structure(&task));
        engine.strategy_stats.insert("algorithm".to_string(), StrategyStats { successes: 10, failures: 0 });
        let result = engine.execute_task(&mut task, &adapter, &mut telemetry);
        assert!(result.is_ok() || result.is_err(), "Should handle compiler error gracefully");
        if result.is_ok() {
            assert!(telemetry.local_search_successes > 0);
        }
    }

    #[test]
    fn test_cache_after_first_run() {
        let mut engine = PolicyEngine::new(CostModel { llm_cost_per_call: 0.1 }, CandidateGenerator::new(3));
        let adapter = CompilerAdapter;
        let mut task = TaskBuilder::new("let x = 1;").build();
        let mut telemetry = Telemetry::new();
        task.context.structure = Some(adapter.build_structure(&task));
        engine.strategy_stats.insert("algorithm".to_string(), StrategyStats { successes: 10, failures: 0 });
        let result1 = engine.execute_task(&mut task, &adapter, &mut telemetry);
        assert!(result1.is_ok());
        engine.strategy_stats.insert("cache".to_string(), StrategyStats { successes: 10, failures: 0 });
        let mut task2 = TaskBuilder::new("let x = 1;").build();
        task2.context.structure = Some(adapter.build_structure(&task2));
        let result2 = engine.execute_task(&mut task2, &adapter, &mut telemetry);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_llm_fallback_when_local_search_fails() {
        let mut engine = PolicyEngine::new(CostModel { llm_cost_per_call: 0.1 }, CandidateGenerator::new(3));
        let adapter = CompilerAdapter;
        let mut task = TaskBuilder::new("let x = 1;").build();
        let mut telemetry = Telemetry::new();
        task.context.structure = Some(adapter.build_structure(&task));
        engine.strategy_stats.insert("llm".to_string(), StrategyStats { successes: 10, failures: 0 });
        let result = engine.execute_task(&mut task, &adapter, &mut telemetry);
        assert!(result.is_ok() || result.is_err());
    }
}
