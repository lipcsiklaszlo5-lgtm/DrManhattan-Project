#[cfg(test)]
mod tests {
    use crate::abstraction::representation::RepresentationFactory;
    use crate::adapter::arc::adapter::ArcGrid;

    fn make_grid(w: u8, h: u8, coords: &[(usize, usize, u8)]) -> ArcGrid {
        let mut pixels = vec![0u8; (w as usize) * (h as usize)];
        for &(x, y, color) in coords {
            pixels[y * w as usize + x] = color;
        }
        ArcGrid::new(w, h, pixels)
    }

    #[test]
    fn test_all_representations() {
        let factory = RepresentationFactory::new();
        let grid = make_grid(3, 3, &[(0, 0, 1), (1, 1, 2)]);
        let reps = factory.build_all(&grid);
        assert_eq!(reps.len(), 4);
        for rep in &reps {
            assert!(!rep.graph.nodes.is_empty(), "Representation '{}' has empty graph", rep.name);
        }
    }

    #[test]
    fn test_color_graph_not_empty() {
        let factory = RepresentationFactory::new();
        let grid = make_grid(2, 2, &[(0, 0, 1)]);
        let reps = factory.build_all(&grid);
        let color_rep = reps.iter().find(|r| r.name == "color").unwrap();
        let object_nodes: Vec<_> = color_rep.graph.nodes.iter().filter(|n| n.node_type == "arc_object").collect();
        assert!(!object_nodes.is_empty());
    }

    #[test]
    fn test_symmetry_detection() {
        let factory = RepresentationFactory::new();
        let grid = make_grid(3, 3, &[(0, 0, 1), (0, 2, 1), (2, 0, 2), (2, 2, 2)]);
        let reps = factory.build_all(&grid);
        let sym_rep = reps.iter().find(|r| r.name == "symmetry").unwrap();
        assert!(!sym_rep.graph.nodes.is_empty());
    }
}
