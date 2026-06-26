#[cfg(test)]
mod tests {
    use crate::structure::KernelStructureGraph;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_graph_new_is_empty() {
        let g = KernelStructureGraph::new();
        assert!(g.nodes.is_empty());
        assert!(g.edges.is_empty());
    }

    #[test]
    fn test_add_node_and_edge() {
        let mut g = KernelStructureGraph::new();
        g.add_node("n1", "error");
        g.add_edge("n1", "main_fn", "occurs_in");
        assert_eq!(g.nodes.len(), 1);
        assert_eq!(g.nodes[0].id, "n1");
        assert_eq!(g.nodes[0].node_type, "error");
        assert_eq!(g.edges.len(), 1);
        assert_eq!(g.edges[0].from, "n1");
        assert_eq!(g.edges[0].to, "main_fn");
        assert_eq!(g.edges[0].rel_type, "occurs_in");
    }

    #[test]
    fn test_fingerprint_consistent() {
        let mut g1 = KernelStructureGraph::new();
        g1.add_node("a", "type_a");
        g1.add_edge("a", "b", "depends_on");
        let mut g2 = KernelStructureGraph::new();
        g2.add_node("a", "type_a");
        g2.add_edge("a", "b", "depends_on");
        assert_eq!(g1.fingerprint(), g2.fingerprint());
    }

    #[test]
    fn test_fingerprint_different() {
        let mut g1 = KernelStructureGraph::new();
        g1.add_node("a", "type_a");
        g1.add_edge("a", "b", "depends_on");
        let mut g2 = KernelStructureGraph::new();
        g2.add_node("a", "type_a");
        // extra edge
        g2.add_edge("a", "c", "references");
        assert_ne!(g1.fingerprint(), g2.fingerprint());
    }
}
