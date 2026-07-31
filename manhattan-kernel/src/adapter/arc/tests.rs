#[cfg(test)]
mod tests {
    use super::super::adapter::ArcGrid;
    use super::super::ArcAdapter;

    #[test]
    fn test_grid_to_ksg_and_back() {
        let pixels = vec![
            1, 1, 0,
            1, 0, 2,
            0, 2, 2,
        ];
        let grid = ArcGrid::new(3, 3, pixels);
        let ksg = ArcAdapter::grid_to_ksg(&grid);
        assert!(ksg.nodes.len() >= 2, "Expected at least 2 nodes, got {}", ksg.nodes.len());
        let reconstructed = ArcAdapter::ksg_to_grid(&ksg, 3, 3, 0);
        assert_eq!(grid.pixels, reconstructed.pixels, "Grid reconstruction failed");
    }

    #[test]
    fn test_empty_grid() {
        let pixels = vec![0; 9];
        let grid = ArcGrid::new(3, 3, pixels);
        let ksg = ArcAdapter::grid_to_ksg(&grid);
        assert!(ksg.nodes.is_empty(), "Expected no nodes for empty grid");
        let reconstructed = ArcAdapter::ksg_to_grid(&ksg, 3, 3, 0);
        assert_eq!(grid.pixels, reconstructed.pixels, "Empty grid reconstruction failed");
    }

    #[test]
    fn test_single_object() {
        let pixels = vec![
            1, 1, 0,
            1, 1, 0,
            0, 0, 0,
        ];
        let grid = ArcGrid::new(3, 3, pixels);
        let ksg = ArcAdapter::grid_to_ksg(&grid);
        assert_eq!(ksg.nodes.len(), 1, "Expected exactly 1 node");
        let node = &ksg.nodes[0];
        assert_eq!(node.attributes.get("color").unwrap(), "1");
        assert_eq!(node.attributes.get("bbox_x").unwrap(), "0");
        assert_eq!(node.attributes.get("bbox_y").unwrap(), "0");
        assert_eq!(node.attributes.get("bbox_w").unwrap(), "2");
        assert_eq!(node.attributes.get("bbox_h").unwrap(), "2");
        let reconstructed = ArcAdapter::ksg_to_grid(&ksg, 3, 3, 0);
        assert_eq!(grid.pixels, reconstructed.pixels);
    }
}
