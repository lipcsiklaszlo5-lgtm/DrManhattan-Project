#[cfg(test)]
mod arc_compose_tests {
    use manhattan_kernel::adapter::arc::adapter::ArcGrid;
    use manhattan_kernel::adapter::arc::ArcAdapter;
    use manhattan_kernel::policy::{CostModel, PolicyEngine};
    use manhattan_kernel::candidate::CandidateGenerator;
    use manhattan_kernel::task::{Task, TaskType};
    use manhattan_kernel::telemetry::Telemetry;
    use manhattan_kernel::adapter::compiler::CompilerAdapter;

    #[test]
    fn test_compose_translate_recolor_rotate() {
        // Bemenet: 2x1 piros téglalap a bal felső sarokban
        let input_pixels = vec![
            1, 1, 0,
            0, 0, 0,
            0, 0, 0,
        ];
        // Cél: 1x2 kék téglalap a jobb alsó sarokban (forgatás + átszínezés + mozgatás)
        let target_pixels = vec![
            0, 0, 0,
            0, 2, 0,
            0, 2, 0,
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

        // Betanítás: a kernel tanulja meg a komplex transzformációt
        engine.program_synthesizer.learn_from_example(&input_ksg, &target_ksg);
        println!("Programs after training: {}", engine.program_synthesizer.programs.len());
        for (i, p) in engine.program_synthesizer.programs.iter().enumerate() {
            println!("  Program {}: {} steps, confidence: {}", i, p.steps.len(), p.confidence);
            for step in &p.steps {
                println!("    {:?}", step);
            }
        }

        let mut task = Task::builder("compose translate recolor rotate test")
            .task_type(TaskType::Transformation)
            .grid(input_grid)
            .target_grid(target_grid)
            .build();

        let result = engine.execute_task(&mut task, &adapter, &mut telemetry);
        assert!(result.is_ok(), "Compose task should succeed, got: {:?}", result.err());

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

        // Ellenőrizzük, hogy a program tartalmazza mindhárom operátort
        if let Some(program) = best_hypothesis.unwrap().program.as_ref() {
            let has_rotate = program.steps.iter().any(|s| matches!(s, manhattan_kernel::sandbox::operators::Transformation::Rotate { .. }));
            let has_translate = program.steps.iter().any(|s| matches!(s, manhattan_kernel::sandbox::operators::Transformation::Translate { .. }));
            let has_recolor = program.steps.iter().any(|s| matches!(s, manhattan_kernel::sandbox::operators::Transformation::Recolor { .. }));
            assert!(has_rotate, "Must contain Rotate");
            assert!(has_translate, "Must contain Translate");
            assert!(has_recolor, "Must contain Recolor");
        }
    }
}
