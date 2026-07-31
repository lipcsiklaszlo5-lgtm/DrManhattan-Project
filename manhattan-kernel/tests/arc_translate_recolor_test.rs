#[cfg(test)]
mod arc_translate_recolor_tests {
    use manhattan_kernel::adapter::arc::adapter::ArcGrid;
    use manhattan_kernel::adapter::arc::ArcAdapter;
    use manhattan_kernel::policy::{CostModel, PolicyEngine};
    use manhattan_kernel::candidate::CandidateGenerator;
    use manhattan_kernel::task::{Task, TaskType};
    use manhattan_kernel::telemetry::Telemetry;
    use manhattan_kernel::adapter::compiler::CompilerAdapter;

    #[test]
    fn test_arc_translate_and_recolor() {
        // Bemenet: piros (1) 2x2-es négyzet a bal felső sarokban
        let input_pixels = vec![
            1, 1, 0,
            1, 1, 0,
            0, 0, 0,
        ];
        // Cél: ugyanaz a négyzet a jobb alsó sarokban, kékre (2) színezve
        let target_pixels = vec![
            0, 0, 0,
            0, 2, 2,
            0, 2, 2,
        ];

        let input_grid = ArcGrid::new(3, 3, input_pixels);
        let target_grid = ArcGrid::new(3, 3, target_pixels);
        let input_ksg = ArcAdapter::grid_to_ksg(&input_grid);
        let target_ksg = ArcAdapter::grid_to_ksg(&target_grid);

        let cost_model = CostModel { llm_cost_per_call: 0.01 };
        let candidate_gen = CandidateGenerator::new(1);
        let mut engine = PolicyEngine::new(cost_model, candidate_gen);
        let adapter = CompilerAdapter;
        let mut telemetry = Telemetry::new();

        // Betanítás: a kernel tanulja meg a Translate + Recolor kombinációt
        engine.program_synthesizer.learn_from_example(&input_ksg, &target_ksg);
        println!("Programs after training: {}", engine.program_synthesizer.programs.len());
        for (i, p) in engine.program_synthesizer.programs.iter().enumerate() {
            println!("  Program {}: {:?} steps, confidence: {}", i, p.steps.len(), p.confidence);
            for step in &p.steps {
                println!("    Step: {:?}", step);
            }
        }

        let mut task = Task::builder("ARC translate + recolor test")
            .task_type(TaskType::Transformation)
            .grid(input_grid)
            .target_grid(target_grid)
            .build();

        let result = engine.execute_task(&mut task, &adapter, &mut telemetry);
        assert!(result.is_ok(), "ARC translate+recolor task should succeed, got: {:?}", result.err());

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
