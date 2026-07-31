#[cfg(test)]
mod arc_generalize_tests {
    use manhattan_kernel::adapter::arc::adapter::ArcGrid;
    use manhattan_kernel::adapter::arc::ArcAdapter;
    use manhattan_kernel::policy::{CostModel, PolicyEngine};
    use manhattan_kernel::candidate::CandidateGenerator;
    use manhattan_kernel::task::{Task, TaskType};
    use manhattan_kernel::telemetry::Telemetry;
    use manhattan_kernel::adapter::compiler::CompilerAdapter;

    #[test]
    fn test_arc_generalize_and_apply() {
        // 1. példa: piros (1) 2x2 → kék (2) 2x2
        let input1 = ArcGrid::new(3, 3, vec![1,1,0, 1,1,0, 0,0,0]);
        let target1 = ArcGrid::new(3, 3, vec![2,2,0, 2,2,0, 0,0,0]);
        let ksg_in1 = ArcAdapter::grid_to_ksg(&input1);
        let ksg_out1 = ArcAdapter::grid_to_ksg(&target1);

        // 2. példa: zöld (3) 2x2 → sárga (4) 2x2
        let input2 = ArcGrid::new(3, 3, vec![3,3,0, 3,3,0, 0,0,0]);
        let target2 = ArcGrid::new(3, 3, vec![4,4,0, 4,4,0, 0,0,0]);
        let ksg_in2 = ArcAdapter::grid_to_ksg(&input2);
        let ksg_out2 = ArcAdapter::grid_to_ksg(&target2);

        let pairs = vec![
            (ksg_in1, ksg_out1),
            (ksg_in2, ksg_out2),
        ];

        let cost_model = CostModel { llm_cost_per_call: 0.01 };
        let candidate_gen = CandidateGenerator::new(1);
        let mut engine = PolicyEngine::new(cost_model, candidate_gen);
        let adapter = CompilerAdapter;
        let mut telemetry = Telemetry::new();

        // Betanítás: általánosítás 2 példából
        engine.program_synthesizer.generalize_from_pairs(&pairs);
        println!("Programs after generalization: {}", engine.program_synthesizer.programs.len());
        for (i, p) in engine.program_synthesizer.programs.iter().enumerate() {
            println!("  Program {}: {} steps", i, p.steps.len());
            for step in &p.steps {
                println!("    {:?}", step);
            }
        }

        // Új bemenet: lila (5) 2x2 → elvárás: sárga (4) 2x2
        let input_new = ArcGrid::new(3, 3, vec![5,5,0, 5,5,0, 0,0,0]);
        let target_new = ArcGrid::new(3, 3, vec![4,4,0, 4,4,0, 0,0,0]);

        let mut task = Task::builder("generalize recolor test")
            .task_type(TaskType::Transformation)
            .grid(input_new)
            .target_grid(target_new)
            .build();

        let result = engine.execute_task(&mut task, &adapter, &mut telemetry);
        assert!(result.is_ok(), "Generalization task should succeed, got: {:?}", result.err());

        println!("Hypotheses count: {}", engine.hypothesis_manager.hypotheses.len());
        for h in &engine.hypothesis_manager.hypotheses {
            println!("  Rep: {}, Score: {:.2}, Has program: {}", 
                h.representation_name, h.score(), h.program.is_some());
            if let Some(ref p) = h.program {
                println!("    Program steps: {:?}", p.steps);
            }
        }

        let best_hypothesis = engine.hypothesis_manager.best_hypothesis();
        assert!(best_hypothesis.is_some(), "Should find a hypothesis with a program");
        println!("Best representation: {:?}", engine.hypothesis_manager.best_representation_name());
        println!("Telemetry: llm_calls={}, local_search_successes={}", telemetry.llm_calls, telemetry.local_search_successes);
    }
}
