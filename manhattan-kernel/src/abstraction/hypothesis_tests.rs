#[cfg(test)]
mod tests {
    use crate::abstraction::hypothesis::{Hypothesis, HypothesisManager};
    use crate::abstraction::program::{Program, ProgramSynthesizer};
    use crate::adapter::arc::adapter::ArcGrid;
    use crate::sandbox::operators::Transformation;

    #[test]
    fn test_hypothesis_scoring() {
        let mut h1 = Hypothesis::new("color".into(), None, 0.5);
        h1.success_count = 3; h1.total_attempts = 5;
        let mut h2 = Hypothesis::new("symmetry".into(), None, 1.0);
        h2.success_count = 1; h2.total_attempts = 5;
        assert!(h1.score() > h2.score(), "h1 should score higher than h2");
    }

    #[test]
    fn test_hypothesis_manager_process_grid() {
        let mut manager = HypothesisManager::new();
        let mut synthesizer = ProgramSynthesizer::new();
        let pixels = vec![0, 0, 0, 0, 1, 0, 0, 0, 0];
        let grid = ArcGrid::new(3, 3, pixels);
        let target_ksg = crate::adapter::arc::adapter::ArcAdapter::grid_to_ksg(&grid);
        manager.process_grid(&grid, &mut synthesizer, Some(&target_ksg));
        assert_eq!(manager.hypotheses.len(), 4, "Should have 4 hypotheses");
    }

    #[test]
    fn test_record_success_updates_stats() {
        let mut manager = HypothesisManager::new();
        let h = Hypothesis::new("color".into(), None, 0.5);
        manager.hypotheses.push(h);
        manager.record_success("color");
        assert_eq!(manager.hypotheses[0].success_count, 1);
        assert_eq!(manager.hypotheses[0].total_attempts, 1);
    }

    #[test]
    fn test_program_cost() {
        let manager = HypothesisManager::new();
        let program = Program::new(vec![Transformation::NoOp]);
        let cost = manager.program_cost(&program);
        assert!(cost >= 0.0);
    }
}
