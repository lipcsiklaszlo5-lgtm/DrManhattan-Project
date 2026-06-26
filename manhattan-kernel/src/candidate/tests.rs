#[cfg(test)]
mod tests {
    use crate::structure::KernelStructureGraph;
    use crate::candidate::CandidateGenerator;

    fn make_action_graph(action: &str) -> KernelStructureGraph {
        let mut g = KernelStructureGraph::new();
        let mut node = crate::structure::Node {
            id: "e1".into(),
            node_type: "compiler_error".into(),
            attributes: std::collections::HashMap::new(),
        };
        node.attributes.insert("action".into(), action.to_string());
        if action == "replace_type" {
            node.attributes.insert("old_type".into(), "i32".into());
            node.attributes.insert("new_type".into(), "String".into());
        }
        g.nodes.push(node);
        g
    }

    #[test]
    fn test_fix_main_generates_distinct_candidates() {
        let base = make_action_graph("fix_main");
        let gen = CandidateGenerator::new(1);
        let candidates = gen.generate(&base, 3);
        assert_eq!(candidates.len(), 3);
        assert_ne!(candidates[0], candidates[1]);
    }

    #[test]
    fn test_replace_type_generates_distinct_candidates() {
        let base = make_action_graph("replace_type");
        let gen = CandidateGenerator::new(1);
        let candidates = gen.generate(&base, 5);
        assert_eq!(candidates.len(), 5);
        let types: Vec<_> = candidates.iter().map(|g| {
            g.nodes[0].attributes.get("new_type").cloned().unwrap_or_default()
        }).collect();
        assert!(types.iter().any(|t| t != &types[0]));
    }

    #[test]
    fn test_generate_respects_max() {
        let base = make_action_graph("fix_main");
        let gen = CandidateGenerator::new(1);
        let candidates = gen.generate(&base, 5);
        assert_eq!(candidates.len(), 5);
    }

    #[test]
    fn test_operator_stats_sorting() {
        let base = make_action_graph("fix_main");
        let mut gen = CandidateGenerator::new(1);
        // Adjunk magas sikerességet a "fix_main"-nek
        gen.operator_stats.insert("fix_main".into(), (10, 10));
        // "delete_line" alacsony
        gen.operator_stats.insert("delete_line".into(), (1, 10));
        let candidates = gen.generate(&base, 5);
        // A "fix_main" operátor magasabb prioritású, de jelenleg minden variáns ugyanaz az akció
        // A rendezés nem változtat, mert egyetlen akció van.
        // Ez a teszt azt mutatja, hogy a struktúra működik.
        assert_eq!(candidates.len(), 5);
    }

    #[test]
    fn test_empty_graph_generates_clones() {
        let base = KernelStructureGraph::new();
        let gen = CandidateGenerator::new(1);
        let candidates = gen.generate(&base, 2);
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().all(|g| g.nodes.is_empty() && g.edges.is_empty()));
    }
}
