#[cfg(test)]
mod tests {
    use crate::abstraction::hypothesis::{Hypothesis, HypothesisManager};
    use crate::abstraction::program::ProgramSynthesizer;
    use crate::adapter::arc::adapter::ArcGrid;
    use crate::structure::KernelStructureGraph;
    use crate::adapter::arc::ArcAdapter;
    use crate::sandbox::operators::Transformation;

    #[test]
    fn test_hypothesis_scoring() {
        let mut h1 = Hypothesis::new("color".into(), KernelStructureGraph::new());
        h1.confidence = 0.8;
        h1.success_count = 8;
        h1.total_attempts = 10;

        let mut h2 = Hypothesis::new("symmetry".into(), KernelStructureGraph::new());
        h2.confidence = 0.5;
        h2.success_count = 1;
        h2.total_attempts = 2;

        assert!(h1.score() > h2.score(), "h1 should score higher than h2");
    }

    #[test]
    fn test_hypothesis_manager_process_grid() {
        let pixels = vec![1, 1, 0, 1, 0, 2, 0, 2, 2];
        let grid = ArcGrid::new(3, 3, pixels);
        let target = ArcAdapter::grid_to_ksg(&grid);

        let mut manager = HypothesisManager::new();
        let mut synthesizer = ProgramSynthesizer::new();

        // Betanítás
        synthesizer.learn_from_example(&target, &target);
        manager.process_grid(&grid, &mut synthesizer, Some(&target));

        assert_eq!(manager.hypotheses.len(), 4, "Should have 4 hypotheses");

        // Adjunk programot az első hipotézishez, hogy a best_hypothesis() megtalálja
        if let Some(h) = manager.hypotheses.first_mut() {
            h.program = Some(crate::abstraction::program::Program::new(vec![Transformation::NoOp]));
        }

        let best = manager.best_hypothesis();
        assert!(best.is_some(), "Should have a best hypothesis");
    }

    #[test]
    fn test_record_success_updates_stats() {
        let mut manager = HypothesisManager::new();
        let pixels = vec![1, 0, 1, 2, 0, 2, 3, 0, 3];
        let grid = ArcGrid::new(3, 3, pixels);
        let target = ArcAdapter::grid_to_ksg(&grid);
        let mut synthesizer = ProgramSynthesizer::new();

        manager.process_grid(&grid, &mut synthesizer, Some(&target));

        manager.record_success("color");
        manager.record_failure("symmetry");

        let color_stats = manager.representation_stats.get("color").unwrap();
        assert_eq!(color_stats.0, 1, "Color should have 1 success");
        assert_eq!(color_stats.1, 1, "Color should have 1 attempt");

        let sym_stats = manager.representation_stats.get("symmetry").unwrap();
        assert_eq!(sym_stats.0, 0, "Symmetry should have 0 successes");
        assert_eq!(sym_stats.1, 1, "Symmetry should have 1 attempt");
    }

    #[test]
    fn test_program_cost() {
        let manager = HypothesisManager::new();
        let program = crate::abstraction::program::Program::new(vec![
            crate::sandbox::operators::Transformation::Translate {
                node_id: "test".into(), dx: 1, dy: 1
            },
            crate::sandbox::operators::Transformation::Recolor {
                node_id: "test".into(), new_color: "2".into()
            },
        ]);

        let cost = manager.program_cost(&program);
        assert!((cost - 2.0).abs() < 0.001, "Program cost should be 2.0, got {}", cost);
    }
}
