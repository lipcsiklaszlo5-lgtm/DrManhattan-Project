use manhattan_kernel::object_selector::{ObjectSelector, SelectionStrategy};
use manhattan_kernel::predicate::builtin::*;
use manhattan_kernel::structure::KernelStructureGraph;

fn make_graph(specs: Vec<(&str, u64, i64, i64, &str)>) -> KernelStructureGraph {
    let mut g = KernelStructureGraph::new();
    for (id, area, x, y, color) in specs {
        let node = g.add_node(id, "arc_object");
        node.attributes.insert("area".into(), area.to_string());
        node.attributes.insert("bbox_x".into(), x.to_string());
        node.attributes.insert("bbox_y".into(), y.to_string());
        node.attributes.insert("bbox_w".into(), "1".to_string());
        node.attributes.insert("bbox_h".into(), "1".to_string());
        node.attributes.insert("color".into(), color.to_string());
    }
    g
}

#[test]
fn test_ranking_order_matches_score_descending() {
    let g = make_graph(vec![("a", 2, 0, 0, "1"), ("b", 10, 1, 1, "1"), ("c", 5, 2, 2, "1")]);
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::All, None);
    let scores: Vec<f32> = result.ranking.iter().map(|s| s.score).collect();
    // Should be descending (largest first)
    for i in 1..scores.len() {
        assert!(scores[i-1] >= scores[i], "Ranking not in descending score order");
    }
}

#[test]
fn test_ranking_contains_all_nodes() {
    let g = make_graph(vec![("a", 2, 0, 0, "1"), ("b", 10, 1, 1, "1"), ("c", 5, 2, 2, "1")]);
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::All, None);
    assert_eq!(result.ranking.len(), 3);
}

#[test]
fn test_ranking_no_duplicates() {
    let g = make_graph(vec![("a", 2, 0, 0, "1"), ("b", 10, 1, 1, "1")]);
    let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::All, None);
    let ids: Vec<&str> = result.ranking.iter().map(|s| s.node_id.as_str()).collect();
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(ids.len(), unique.len());
}

#[test]
fn test_ranking_stable_across_runs() {
    let g = make_graph(vec![("obj_2", 5, 0, 0, "1"), ("obj_10", 5, 1, 1, "1"), ("obj_1", 5, 2, 2, "1")]);
    let first = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::All, None);
    for _ in 0..50 {
        let next = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::All, None);
        for (a, b) in first.ranking.iter().zip(next.ranking.iter()) {
            assert_eq!(a.node_id, b.node_id);
        }
    }
}
