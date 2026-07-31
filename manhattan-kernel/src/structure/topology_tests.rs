#[cfg(test)]
mod tests {
    use crate::structure::{KernelStructureGraph, Node, Edge};
    use crate::structure::topology::{wl_hash, graph_diff, NodeTransformation};
    use std::collections::HashMap;

    fn make_test_graph() -> KernelStructureGraph {
        let mut g = KernelStructureGraph::new();
        let mut n1 = Node {
            id: "obj_0".to_string(),
            node_type: "arc_object".to_string(),
            attributes: HashMap::new(),
        };
        n1.attributes.insert("color".to_string(), "1".to_string());
        n1.attributes.insert("bbox_x".to_string(), "1".to_string());
        n1.attributes.insert("bbox_y".to_string(), "2".to_string());
        g.nodes.push(n1);

        let mut n2 = Node {
            id: "obj_1".to_string(),
            node_type: "arc_object".to_string(),
            attributes: HashMap::new(),
        };
        n2.attributes.insert("color".to_string(), "2".to_string());
        n2.attributes.insert("bbox_x".to_string(), "4".to_string());
        n2.attributes.insert("bbox_y".to_string(), "5".to_string());
        g.nodes.push(n2);

        g.edges.push(Edge {
            from: "obj_0".to_string(),
            to: "obj_1".to_string(),
            rel_type: "left_of".to_string(),
            attributes: HashMap::new(),
        });
        g
    }

    #[test]
    fn test_wl_hash_deterministic() {
        let g1 = make_test_graph();
        let g2 = make_test_graph();
        assert_eq!(wl_hash(&g1, 3), wl_hash(&g2, 3), "WL hash must be deterministic for identical graphs");
    }

    #[test]
    fn test_wl_hash_different_for_different_graphs() {
        let mut g1 = make_test_graph();
        let g2 = make_test_graph();
        g1.nodes[0].attributes.insert("color".to_string(), "9".to_string());
        assert_ne!(wl_hash(&g1, 3), wl_hash(&g2, 3), "WL hash must differ for different graphs");
    }

    #[test]
    fn test_graph_diff_recolor() {
        let before = make_test_graph();
        let mut after = before.clone();
        after.nodes[0].attributes.insert("color".to_string(), "9".to_string());
        let diffs = graph_diff(&before, &after);
        assert!(diffs.iter().any(|t| matches!(t, NodeTransformation::Recolor { node_id, new_color } if node_id == "obj_0" && new_color == "9")), "Must detect recolor");
    }

    #[test]
    fn test_graph_diff_translate() {
        let before = make_test_graph();
        let mut after = before.clone();
        after.nodes[0].attributes.insert("bbox_x".to_string(), "10".to_string());
        after.nodes[0].attributes.insert("bbox_y".to_string(), "20".to_string());
        let diffs = graph_diff(&before, &after);
        assert!(diffs.iter().any(|t| matches!(t, NodeTransformation::Translate { node_id, dx, dy } if node_id == "obj_0" && *dx == 9 && *dy == 18)), "Must detect translate");
    }

    #[test]
    fn test_graph_diff_create_delete() {
        let before = make_test_graph();
        let after = KernelStructureGraph::new();
        let diffs = graph_diff(&before, &after);
        assert_eq!(diffs.len(), 2, "Must have 2 deletions");
        assert!(diffs.iter().all(|t| matches!(t, NodeTransformation::Delete { .. })));
    }
}
