#[cfg(test)]
mod tests {
    use crate::abstraction::representation::RepresentationFactory;
    use crate::adapter::arc::adapter::ArcGrid;

    #[test]
    fn test_all_representations() {
        let pixels = vec![1,1,0,1,0,2,0,2,2];
        let grid = ArcGrid::new(3,3,pixels);
        let reps = RepresentationFactory::all_representations(&grid);
        assert_eq!(reps.len(), 4);
    }

    #[test]
    fn test_color_graph_not_empty() {
        let pixels = vec![1,2,3,4,5,6,7,8,9];
        let grid = ArcGrid::new(3,3,pixels);
        let g = RepresentationFactory::color_graph(&grid);
        assert!(g.nodes.len() > 0);
    }

    #[test]
    fn test_symmetry_detection() {
        let pixels = vec![1,0,1,2,0,2,3,0,3];
        let grid = ArcGrid::new(3,3,pixels);
        let g = RepresentationFactory::symmetry_graph(&grid);
        assert!(g.nodes.len() > 0);
    }
}
