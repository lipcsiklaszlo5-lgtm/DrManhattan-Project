#[cfg(test)]
mod arc_subgoal_tests {
    use manhattan_kernel::adapter::arc::adapter::ArcGrid;
    use manhattan_kernel::adapter::arc::ArcAdapter;
    use manhattan_kernel::policy::{CostModel, PolicyEngine};
    use manhattan_kernel::candidate::CandidateGenerator;
    use manhattan_kernel::task::{Task, TaskType};
    use manhattan_kernel::telemetry::Telemetry;
    use manhattan_kernel::adapter::compiler::CompilerAdapter;
    use manhattan_kernel::abstraction::goal_decomposer::GoalDecomposer;

    #[test]
    fn test_solve_via_subgoals() {
        let input_pixels = vec![1,1,0, 1,1,0, 0,0,0];
        let target_pixels = vec![0,0,0, 0,2,2, 0,2,2];
        let input_grid = ArcGrid::new(3, 3, input_pixels);
        let target_grid = ArcGrid::new(3, 3, target_pixels.clone());
        let input_ksg = ArcAdapter::grid_to_ksg(&input_grid);
        let target_ksg = ArcAdapter::grid_to_ksg(&target_grid);

        let subgoals = GoalDecomposer::decompose(&input_ksg, &target_ksg);
        println!("Subgoals: {:?}", subgoals.iter().map(|s| &s.description).collect::<Vec<_>>());
        assert!(!subgoals.is_empty(), "Must find at least one subgoal");

        let cost_model = CostModel { llm_cost_per_call: 0.01 };
        let candidate_gen = CandidateGenerator::new(1);
        let mut engine = PolicyEngine::new(cost_model, candidate_gen);
        let adapter = CompilerAdapter;
        let mut telemetry = Telemetry::new();

        engine.program_synthesizer.learn_from_example(&input_ksg, &target_ksg);
        println!("Programs after training: {}", engine.program_synthesizer.programs.len());

        let mut current_grid = input_grid.clone();
        for sg in &subgoals {
            println!("Solving subgoal: {}", sg.description);
            let sg_target_grid = ArcAdapter::ksg_to_grid(&sg.target_ksg, target_grid.width, target_grid.height, 0);

            let mut task = Task::builder("subgoal")
                .task_type(TaskType::Transformation)
                .grid(current_grid.clone())
                .target_grid(sg_target_grid.clone())
                .build();

            let result = engine.execute_task(&mut task, &adapter, &mut telemetry);
            assert!(result.is_ok(), "Subgoal failed: {:?}", result.err());

            // Ha a részcél sikeres, frissítjük a current_grid-et a target_grid-re
            if result.is_ok() {
                current_grid = sg_target_grid;
            }
        }

        assert_eq!(current_grid.pixels, target_pixels, "Final grid must match target after all subgoals");
        println!("All subgoals solved successfully");
    }
}
