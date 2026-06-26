#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::policy::{CostModel, PolicyEngine};
    use crate::candidate::CandidateGenerator;
    use crate::task::{Task, TaskType};
    use crate::adapter::compiler::CompilerAdapter;
    use crate::telemetry::Telemetry;
    use crate::executor::LlmExecutor;
    use crate::structure::{KernelStructureGraph, Node};

    #[test]
    fn test_execute_task_with_compiler_error() {
        let mut engine = PolicyEngine::new(CostModel { llm_cost_per_call: 0.01 }, CandidateGenerator::new(1));
        let adapter = CompilerAdapter;
        let mut telemetry = Telemetry::new();
        let mut task = Task::builder("fn main() { let x: i32 = \"hello\"; }")
            .task_type(TaskType::CodeGeneration)
            .build();

        let result = engine.execute_task(&mut task, &adapter, &mut telemetry);
        assert!(result.is_ok());
        assert!(telemetry.local_search_successes > 0);
        assert_eq!(engine.episodic_log.len(), 1);
        assert!(engine.episodic_log[0].success);
    }

    #[test]
    fn test_execute_task_no_error() {
        let mut engine = PolicyEngine::new(CostModel { llm_cost_per_call: 0.01 }, CandidateGenerator::new(1));
        let adapter = CompilerAdapter;
        let mut telemetry = Telemetry::new();
        let mut task = Task::builder("fn main() {}")
            .task_type(TaskType::CodeGeneration)
            .build();

        let result = engine.execute_task(&mut task, &adapter, &mut telemetry);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "already correct");
        assert_eq!(engine.episodic_log.len(), 1);
        assert!(engine.episodic_log[0].success);
    }

    #[test]
    fn test_cache_after_first_run() {
        let mut engine = PolicyEngine::new(CostModel { llm_cost_per_call: 0.01 }, CandidateGenerator::new(1));
        let adapter = CompilerAdapter;
        let mut telemetry = Telemetry::new();
        let mut task = Task::builder("fn main() { let x: i32 = \"hello\"; }")
            .task_type(TaskType::CodeGeneration)
            .build();

        let _ = engine.execute_task(&mut task, &adapter, &mut telemetry);
        assert!(telemetry.local_search_successes > 0);
        assert_eq!(engine.episodic_log.len(), 1);

        let mut task2 = Task::builder("fn main() { let x: i32 = \"hello\"; }")
            .task_type(TaskType::CodeGeneration)
            .build();
        let mut telemetry2 = Telemetry::new();
        let result = engine.execute_task(&mut task2, &adapter, &mut telemetry2);
        assert!(result.is_ok());
        assert!(telemetry2.cache_hits > 0);
        assert_eq!(engine.episodic_log.len(), 2);
        assert!(engine.episodic_log[1].success);
    }

    #[test]
    fn test_llm_fallback_when_local_search_fails() {
        let llm = LlmExecutor::new().with_mock_response("fn main() {}".to_string());
        let mut engine = PolicyEngine::new(CostModel { llm_cost_per_call: 0.01 }, CandidateGenerator::new(1))
            .with_llm_executor(&llm);
        let adapter = CompilerAdapter;
        let mut telemetry = Telemetry::new();

        // Create a task with a pre-built structure that has an unknown action
        let mut task = Task::builder("doesn't matter").task_type(TaskType::CodeGeneration).build();
        let mut g = KernelStructureGraph::new();
        let mut node = Node {
            id: "err1".into(),
            node_type: "compiler_error".into(),
            attributes: HashMap::new(),
        };
        node.attributes.insert("action".into(), "unknown".into());
        g.nodes.push(node);
        task.context.structure = Some(g);

        let result = engine.execute_task(&mut task, &adapter, &mut telemetry);
        // The LLM response "fn main() {}" is valid, so the validation should pass
        assert!(result.is_ok());
        // LLM call must have been recorded
        assert!(telemetry.llm_calls > 0);
    }
}
