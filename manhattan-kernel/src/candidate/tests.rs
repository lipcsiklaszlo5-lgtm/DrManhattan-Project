#[cfg(test)]
mod tests {
    use crate::structure::KernelStructureGraph;
    use crate::candidate::CandidateGenerator;

    #[test]
    fn test_generate_deletes_edge() {
        let mut base = KernelStructureGraph::new();
        base.add_node("a", "func");
        base.add_node("b", "error");
        base.add_edge("a", "b", "causes");
        let gen = CandidateGenerator::new(1);
        let candidates = gen.generate(&base, 5);
        assert!(!candidates.is_empty());
        assert!(candidates[0].edges.is_empty());
    }

    #[test]
    fn test_generate_respects_max() {
        let mut base = KernelStructureGraph::new();
        base.add_node("a", "func");
        base.add_node("b", "error");
        base.add_edge("a", "b", "causes");
        base.add_edge("b", "a", "fixes");
        let gen = CandidateGenerator::new(1);
        let candidates = gen.generate(&base, 1);
        assert_eq!(candidates.len(), 1);
    }

    #[test]
    fn test_generate_no_edges_produces_empty() {
        let base = KernelStructureGraph::new();
        let gen = CandidateGenerator::new(1);
        let candidates = gen.generate(&base, 5);
        assert!(candidates.is_empty());
    }
}
