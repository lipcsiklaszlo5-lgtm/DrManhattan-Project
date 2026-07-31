#[cfg(test)]
mod unit_program_synthesizer_tests {
    use manhattan_kernel::adapter::arc::adapter::ArcGrid;
    use manhattan_kernel::adapter::arc::ArcAdapter;
    use manhattan_kernel::abstraction::program::ProgramSynthesizer;

    #[test]
    fn test_learn_multi_operator() {
        let input_pixels = vec![
            1, 1, 0, 0,
            1, 1, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let target_pixels = vec![
            0, 0, 0, 0,
            0, 2, 2, 0,
            0, 2, 2, 1,
            1, 0, 0, 0,
        ];

        let input_grid = ArcGrid::new(4, 4, input_pixels);
        let target_grid = ArcGrid::new(4, 4, target_pixels);
        let input_ksg = ArcAdapter::grid_to_ksg(&input_grid);
        let target_ksg = ArcAdapter::grid_to_ksg(&target_grid);

        let mut synthesizer = ProgramSynthesizer::new();
        let program = synthesizer.learn_from_example(&input_ksg, &target_ksg);

        assert!(program.is_some(), "Must learn a program for multi-operator task");
        let program = program.unwrap();

        println!("Learned program steps ({} total):", program.steps.len());
        for step in &program.steps {
            println!("  {:?}", step);
        }

        // A program alkalmazása pixel-perfect kell legyen
        let result_graph = program.apply(&input_ksg);
        let result_grid = ArcAdapter::ksg_to_grid(&result_graph, 4, 4, 0);
        assert_eq!(result_grid.pixels, target_grid.pixels, "Program must produce the exact target grid");

        // Ellenőrizzük, hogy a szükséges operátorok mind jelen vannak
        let has_recolor = program.steps.iter().any(|s| matches!(s, manhattan_kernel::sandbox::operators::Transformation::Recolor { .. }));
        let has_translate = program.steps.iter().any(|s| matches!(s, manhattan_kernel::sandbox::operators::Transformation::Translate { .. }));
        let has_create = program.steps.iter().any(|s| matches!(s, manhattan_kernel::sandbox::operators::Transformation::Create { .. }));

        assert!(has_recolor, "Must contain Recolor operator");
        assert!(has_translate, "Must contain Translate operator");
        assert!(has_create, "Must contain Create operator");

        println!("All required operators found: Recolor={}, Translate={}, Create={}", has_recolor, has_translate, has_create);
    }
}
