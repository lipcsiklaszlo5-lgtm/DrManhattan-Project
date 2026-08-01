#[cfg(test)]
mod unit_abstract_recolor_tests {
    use manhattan_kernel::adapter::arc::adapter::ArcGrid;
    use manhattan_kernel::adapter::arc::ArcAdapter;
    use manhattan_kernel::abstraction::program::ProgramSynthesizer;
    use manhattan_kernel::sandbox::operators::Transformation;

    #[test]
    fn test_abstract_recolor() {
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

        let mut synthesizer = ProgramSynthesizer::new();
        let program = synthesizer.generalize_from_pairs(&pairs);
        assert!(program.is_some(), "Must generalize abstract recolor");
        let program = program.unwrap();
        println!("Abstract program steps:");
        for step in &program.steps {
            println!("  {:?}", step);
        }

        // Ellenőrizzük, hogy a program RecolorToTarget-et tartalmaz
        let has_abstract = program.steps.iter().any(|s| matches!(s, crate::sandbox::operators::Transformation::RecolorToTarget { .. }));
        assert!(has_abstract, "Program must contain RecolorToTarget (abstract recolor)");
        println!("Abstraction successful: kernel understands 'change color' as an invariant");
    }
}
