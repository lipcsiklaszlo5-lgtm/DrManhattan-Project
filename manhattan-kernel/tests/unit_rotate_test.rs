#[cfg(test)]
mod unit_rotate_tests {
    use manhattan_kernel::adapter::arc::adapter::ArcGrid;
    use manhattan_kernel::adapter::arc::ArcAdapter;
    use manhattan_kernel::abstraction::program::ProgramSynthesizer;
    use manhattan_kernel::sandbox::operators::Transformation;

    #[test]
    fn test_learn_rotate_detection() {
        let input_pixels = vec![1,1,0, 0,0,0, 0,0,0];
        let target_pixels = vec![1,0,0, 1,0,0, 0,0,0];
        let input_grid = ArcGrid::new(3, 3, input_pixels);
        let target_grid = ArcGrid::new(3, 3, target_pixels.clone());
        let input_ksg = ArcAdapter::grid_to_ksg(&input_grid);
        let target_ksg = ArcAdapter::grid_to_ksg(&target_grid);

        let mut synthesizer = ProgramSynthesizer::new();
        let program = synthesizer.learn_from_example(&input_ksg, &target_ksg);

        assert!(program.is_some(), "Must learn rotate program");
        let program = program.unwrap();
        println!("Learned program steps:");
        for step in &program.steps { println!("  {:?}", step); }

        let has_rotate = program.steps.iter().any(|s| matches!(s, Transformation::Rotate { .. }));
        assert!(has_rotate, "Program must contain Rotate operator");
        println!("Rotate detection successful");
    }
}
