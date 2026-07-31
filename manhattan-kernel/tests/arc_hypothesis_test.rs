#[cfg(test)]
mod arc_hypothesis_tests {
    use manhattan_kernel::adapter::arc::adapter::ArcGrid;
    use manhattan_kernel::adapter::arc::ArcAdapter;
    use manhattan_kernel::abstraction::hypothesis::HypothesisManager;
    use manhattan_kernel::abstraction::program::ProgramSynthesizer;

    #[test]
    fn test_hypothesis_finds_program() {
        let input_pixels = vec![1,1,0, 1,1,0, 0,0,0];
        let target_pixels = vec![2,2,0, 2,2,0, 0,0,0];
        let input_grid = ArcGrid::new(3, 3, input_pixels);
        let target_grid = ArcGrid::new(3, 3, target_pixels);
        let input_ksg = ArcAdapter::grid_to_ksg(&input_grid);
        let target_ksg = ArcAdapter::grid_to_ksg(&target_grid);

        let mut synthesizer = ProgramSynthesizer::new();
        synthesizer.learn_from_example(&input_ksg, &target_ksg);
        println!("Programs count: {}", synthesizer.programs.len());
        for (i, p) in synthesizer.programs.iter().enumerate() {
            println!("  Program {}: {:?} steps, confidence: {}", i, p.steps.len(), p.confidence);
        }

        let mut manager = HypothesisManager::new();
        manager.process_grid(&input_grid, &mut synthesizer, Some(&target_ksg));

        println!("Hypotheses count: {}", manager.hypotheses.len());
        for h in &manager.hypotheses {
            println!("  Rep: {}, Score: {:.2}, Has program: {}",
                h.representation_name, h.score(), h.program.is_some());
            if let Some(ref p) = h.program {
                println!("    Program steps: {:?}", p.steps);
            }
        }

        // A find_best_program tesztelése közvetlenül
        let best = synthesizer.find_best_program(&input_ksg, &target_ksg);
        println!("find_best_program result: {}", best.is_some());

        assert!(best.is_some(), "find_best_program must find the program");
        assert!(!manager.hypotheses.is_empty(), "HypothesisManager must have hypotheses");
        assert!(manager.best_hypothesis().is_some(), "best_hypothesis must return a hypothesis");
    }
}
