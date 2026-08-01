#[cfg(test)]
mod tests {
    use crate::sandbox::synthetic::SyntheticArcGenerator;

    #[test]
    fn test_generate_and_solve_synthetic_task() {
        let mut generator = SyntheticArcGenerator::new();
        let (input, output, _ops) = generator.generate_task(5, 5, 2, 3);
        // Egyszerű ellenőrzés, hogy kaptunk-e valid gridet
        assert!(input.width > 0);
        assert_eq!(input.width, output.width);
        assert_eq!(input.height, output.height);
    }
}
