#[cfg(test)]
mod tests {
    use crate::sandbox::synthetic::SyntheticArcGenerator;
    use crate::adapter::arc::ArcAdapter;
    use crate::abstraction::program::ProgramSynthesizer;

    #[test]
    fn test_generate_and_solve_synthetic_task() {
        let generator = SyntheticArcGenerator::new(42, 3, 2, 5);
        let (input, output, program, input_ksg) = generator.generate_task();
        
        println!("Input grid: {:?}", input.pixels);
        println!("Output grid: {:?}", output.pixels);
        println!("Program: {:?}", program.steps);
        
        // Ellenőrizzük, hogy a program valóban megváltoztatja a rácsot
        if input.pixels == output.pixels {
            println!("Program did not change the grid, skipping test");
            return;
        }
        
        let result_graph = program.apply(&input_ksg);
        let result_grid = ArcAdapter::ksg_to_grid(&result_graph, 5, 5, 0);
        assert_eq!(result_grid.pixels, output.pixels, "Generated program must solve the task");
        
        let mut synthesizer = ProgramSynthesizer::new();
        let output_ksg = ArcAdapter::grid_to_ksg(&output);
        let learned = synthesizer.learn_from_example(&input_ksg, &output_ksg);
        assert!(learned.is_some(), "Synthesizer must learn from synthetic task");
        println!("Learned program: {:?}", learned.unwrap().steps);
    }
}
