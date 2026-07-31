#[cfg(test)]
mod tests {
    use crate::structure::KernelStructureGraph;
    use crate::sandbox::operators::{Transformation, apply_transformation, simulate_plan};

    fn make_test_graph() -> KernelStructureGraph {
        let mut g = KernelStructureGraph::new();
        let node = g.add_node("obj_0", "arc_object");
        node.attributes.insert("color".to_string(), "1".to_string());
        node.attributes.insert("bbox_x".to_string(), "0".to_string());
        node.attributes.insert("bbox_y".to_string(), "0".to_string());
        node.attributes.insert("bbox_w".to_string(), "2".to_string());
        node.attributes.insert("bbox_h".to_string(), "2".to_string());
        g
    }

    #[test]
    fn test_translate() {
        let g = make_test_graph();
        let t = Transformation::Translate { node_id: "obj_0".to_string(), dx: 3, dy: 4 };
        let result = apply_transformation(&g, &t);
        let node = &result.nodes[0];
        assert_eq!(node.attributes.get("bbox_x").unwrap(), "3");
        assert_eq!(node.attributes.get("bbox_y").unwrap(), "4");
    }

    #[test]
    fn test_recolor() {
        let g = make_test_graph();
        let t = Transformation::Recolor { node_id: "obj_0".to_string(), new_color: "5".to_string() };
        let result = apply_transformation(&g, &t);
        assert_eq!(result.nodes[0].attributes.get("color").unwrap(), "5");
    }

    #[test]
    fn test_delete() {
        let g = make_test_graph();
        let t = Transformation::Delete { node_id: "obj_0".to_string() };
        let result = apply_transformation(&g, &t);
        assert!(result.nodes.is_empty());
    }

    #[test]
    fn test_create() {
        let g = make_test_graph();
        let t = Transformation::Create { color: "3".to_string(), bbox_x: 5, bbox_y: 6, bbox_w: 2, bbox_h: 3 };
        let result = apply_transformation(&g, &t);
        assert_eq!(result.nodes.len(), 2);
        let new_node = &result.nodes[1];
        assert_eq!(new_node.attributes.get("color").unwrap(), "3");
        assert_eq!(new_node.attributes.get("bbox_x").unwrap(), "5");
        assert_eq!(new_node.attributes.get("bbox_y").unwrap(), "6");
    }

    #[test]
    fn test_merge() {
        let mut g = make_test_graph();
        let node2 = g.add_node("obj_1", "arc_object");
        node2.attributes.insert("color".to_string(), "2".to_string());
        node2.attributes.insert("bbox_x".to_string(), "3".to_string());
        node2.attributes.insert("bbox_y".to_string(), "3".to_string());
        node2.attributes.insert("bbox_w".to_string(), "2".to_string());
        node2.attributes.insert("bbox_h".to_string(), "2".to_string());

        let t = Transformation::Merge { node_a: "obj_0".to_string(), node_b: "obj_1".to_string() };
        let result = apply_transformation(&g, &t);
        assert_eq!(result.nodes.len(), 1);
    }

    #[test]
    fn test_simulate_plan() {
        let g = make_test_graph();
        let plan = vec![
            Transformation::Translate { node_id: "obj_0".to_string(), dx: 1, dy: 1 },
            Transformation::Recolor { node_id: "obj_0".to_string(), new_color: "9".to_string() },
        ];
        let states = simulate_plan(&g, &plan);
        assert_eq!(states.len(), 3); // eredeti + 2 transzformált állapot
        let final_state = &states[2];
        assert_eq!(final_state.nodes[0].attributes.get("color").unwrap(), "9");
        assert_eq!(final_state.nodes[0].attributes.get("bbox_x").unwrap(), "1");
        assert_eq!(final_state.nodes[0].attributes.get("bbox_y").unwrap(), "1");
    }
}
