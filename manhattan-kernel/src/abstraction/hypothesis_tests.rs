#[cfg(test)]
mod tests {
    use crate::abstraction::hypothesis::{Hypothesis, HypothesisManager};
    use crate::abstraction::program::ProgramSynthesizer;
    use crate::adapter::arc::adapter::ArcGrid;
    use crate::structure::KernelStructureGraph;
    use crate::adapter::arc::ArcAdapter;

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

        // h1-nek magasabb pontszámúnak kell lennie
        assert!(h1.score() > h2.score(), "h1 should score higher than h2");
    }

    #[test]
    fn test_hypothesis_manager_process_grid() {
        let pixels = vec![1, 1, 0, 1, 0, 2, 0, 2, 2];
        let grid = ArcGrid::new(3, 3, pixels);
        let target = ArcAdapter::grid_to_ksg(&grid);

        let mut manager = HypothesisManager::new();
        let mut synthesizer = ProgramSynthesizer::new();

        manager.process_grid(&grid, &mut synthesizer, Some(&target));

        // Kell, hogy legyen 4 hipotézisünk
        assert_eq!(manager.hypotheses.len(), 4, "Should have 4 hypotheses");
        // A legjobb hipotézisnek léteznie kell
        assert!(manager.best_hypothesis().is_some(), "Should have a best hypothesis");
        // A legjobb reprezentáció neve nem üres
        assert!(manager.best_representation_name().is_some(), "Should have a best representation name");
    }

    #[test]
    fn test_record_success_updates_stats() {
        let mut manager = HypothesisManager::new();
        let pixels = vec![1, 0, 1, 2, 0, 2, 3, 0, 3];
        let grid = ArcGrid::new(3, 3, pixels);
        let target = ArcAdapter::grid_to_ksg(&grid);
        let mut synthesizer = ProgramSynthesizer::new();

        manager.process_grid(&grid, &mut synthesizer, Some(&target));

        // Szimuláljunk egy sikeres végrehajtást
        manager.record_success("color");
        manager.record_failure("symmetry");

        // Ellenőrizzük a statisztikákat
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
        // Translate(1.0) + Recolor(1.0) = 2.0
        assert!((cost - 2.0).abs() < 0.001, "Program cost should be 2.0, got {}", cost);
    }
}
