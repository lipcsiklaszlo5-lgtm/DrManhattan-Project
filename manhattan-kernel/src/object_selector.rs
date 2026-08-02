use crate::predicate::{Predicate, PredicateResult};
use crate::structure::KernelStructureGraph;
use std::collections::HashMap;

/// Kiválasztási stratégia.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionStrategy {
    Best,
    TopK(usize),
    Threshold(f32),
    All,
    Unique,
}

/// Pontozási függvény típusa. A bemenet a node és a predikátum-konfidencia.
pub type ScoreFn = fn(&crate::structure::Node, f32) -> f32;

/// Egy kiválasztott objektum metaadata.
#[derive(Debug, Clone)]
pub struct SelectedObject {
    pub node_id: String,
    pub score: f32,
    pub reason: String,
}

/// A kiválasztás eredménye.
#[derive(Debug, Clone)]
pub struct SelectionResult {
    pub selected: Vec<SelectedObject>,
    pub ambiguity: bool,
    pub confidence: f32,
}

impl SelectionResult {
    pub fn best_id(&self) -> Option<&str> {
        self.selected.first().map(|s| s.node_id.as_str())
    }
    pub fn is_unique(&self) -> bool {
        self.selected.len() == 1 && !self.ambiguity
    }
}

pub struct ObjectSelector;

impl ObjectSelector {
    pub fn default_score_fn(node: &crate::structure::Node, predicate_score: f32) -> f32 {
        let area: f32 = node
            .attributes
            .get("area")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.0);
        predicate_score * (1.0 + area.log10().max(0.0))
    }

    pub fn select(
        predicate: &dyn Predicate,
        graph: &KernelStructureGraph,
        strategy: &SelectionStrategy,
        score_fn: Option<ScoreFn>,
    ) -> SelectionResult {
        let score_fn = score_fn.unwrap_or(Self::default_score_fn);

        // 1. Predikátum kiértékelése
        let candidates = match predicate.evaluate(graph) {
            PredicateResult::RankedList(list) => list,
            PredicateResult::Bool(true) => graph
                .nodes
                .iter()
                .map(|n| (n.id.clone(), 1.0))
                .collect(),
            PredicateResult::Bool(false) => Vec::new(),
        };

        if candidates.is_empty() {
            return SelectionResult {
                selected: Vec::new(),
                ambiguity: false,
                confidence: 0.0,
            };
        }

        // 2. Pontozás
        let node_map: HashMap<&str, &crate::structure::Node> = graph
            .nodes
            .iter()
            .map(|n| (n.id.as_str(), n))
            .collect();

        let mut scored: Vec<SelectedObject> = candidates
            .into_iter()
            .filter_map(|(id, pred_score)| {
                node_map.get(id.as_str()).map(|&node| {
                    let score = score_fn(node, pred_score);
                    SelectedObject {
                        node_id: id.clone(),
                        score,
                        reason: predicate.name().to_string(),
                    }
                })
            })
            .collect();

        // 3. Rangsorolás (stabil rendezés, csökkenő score, determinisztikus tie-break)
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.node_id.cmp(&b.node_id))
        });

        // 4. Stratégia alkalmazása
        match strategy {
            SelectionStrategy::Best => {
                let best_score = scored.first().map(|s| s.score).unwrap_or(0.0);
                let best: Vec<SelectedObject> = scored
                    .into_iter()
                    .take_while(|s| s.score == best_score)
                    .collect();
                let ambiguity = best.len() > 1;
                if ambiguity {
                    let mut best = best;
                    best.sort_by(|a, b| a.node_id.cmp(&b.node_id));
                    best.truncate(1);
                    let confidence = best.first().map(|s| s.score).unwrap_or(0.0);
                    SelectionResult {
                        selected: best,
                        ambiguity: true,
                        confidence,
                    }
                } else {
                    SelectionResult {
                        selected: best,
                        ambiguity: false,
                        confidence: best_score,
                    }
                }
            }
            SelectionStrategy::TopK(k) => {
                let top: Vec<SelectedObject> = scored.into_iter().take(*k).collect();
                let confidence = top.first().map(|s| s.score).unwrap_or(0.0);
                SelectionResult {
                    selected: top,
                    ambiguity: false,
                    confidence,
                }
            }
            SelectionStrategy::Threshold(t) => {
                let filtered: Vec<SelectedObject> = scored
                    .into_iter()
                    .filter(|s| s.score >= *t)
                    .collect();
                let confidence = filtered.first().map(|s| s.score).unwrap_or(0.0);
                SelectionResult {
                    selected: filtered,
                    ambiguity: false,
                    confidence,
                }
            }
            SelectionStrategy::All => {
                let confidence = scored.first().map(|s| s.score).unwrap_or(0.0);
                SelectionResult {
                    selected: scored,
                    ambiguity: false,
                    confidence,
                }
            }
            SelectionStrategy::Unique => {
                let count = scored.len();
                if count == 1 {
                    let confidence = scored[0].score;
                    SelectionResult {
                        selected: scored,
                        ambiguity: false,
                        confidence,
                    }
                } else {
                    let best = scored.into_iter().next().unwrap();
                    SelectionResult {
                        selected: vec![best],
                        ambiguity: true,
                        confidence: 0.0,
                    }
                }
            }
        }
    }

    pub fn select_best_id(
        predicate: &dyn Predicate,
        graph: &KernelStructureGraph,
    ) -> Option<String> {
        let result = Self::select(predicate, graph, &SelectionStrategy::Best, None);
        result.best_id().map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicate::builtin::*;
    use crate::structure::KernelStructureGraph;

    fn make_graph(objects: Vec<(&str, u64, i64, i64, &str)>) -> KernelStructureGraph {
        let mut g = KernelStructureGraph::new();
        for (id, area, x, y, color) in objects {
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
    fn test_best_strategy_selects_largest() {
        let g = make_graph(vec![
            ("a", 5, 0, 0, "1"),
            ("b", 8, 1, 1, "1"),
            ("c", 3, 2, 2, "2"),
        ]);
        let pred = LargestPredicate;
        let result = ObjectSelector::select(&pred, &g, &SelectionStrategy::Best, None);
        assert_eq!(result.selected.len(), 1);
        assert_eq!(result.selected[0].node_id, "b");
        assert!(!result.ambiguity);
    }

    #[test]
    fn test_topk_strategy() {
        let g = make_graph(vec![
            ("a", 5, 0, 0, "1"),
            ("b", 8, 1, 1, "1"),
            ("c", 3, 2, 2, "2"),
            ("d", 7, 3, 3, "3"),
        ]);
        let result = ObjectSelector::select(
            &ColorPredicate { color: "1".into() },
            &g,
            &SelectionStrategy::TopK(2),
            None,
        );
        assert_eq!(result.selected.len(), 2);
    }

    #[test]
    fn test_threshold_strategy() {
        let g = make_graph(vec![
            ("a", 5, 0, 0, "1"),
            ("b", 8, 1, 1, "1"),
        ]);
        let result = ObjectSelector::select(
            &LargestPredicate,
            &g,
            &SelectionStrategy::Threshold(0.9),
            None,
        );
        assert!(!result.selected.is_empty());
    }

    #[test]
    fn test_unique_strategy_with_ambiguity() {
        let g = make_graph(vec![
            ("a", 5, 0, 0, "1"),
            ("b", 5, 1, 1, "2"),
        ]);
        let result = ObjectSelector::select(
            &LargestPredicate,
            &g,
            &SelectionStrategy::Unique,
            None,
        );
        assert!(result.ambiguity);
        assert_eq!(result.selected.len(), 1);
    }

    #[test]
    fn test_determinism() {
        let g = make_graph(vec![
            ("a", 5, 0, 0, "1"),
            ("b", 8, 1, 1, "1"),
            ("c", 3, 2, 2, "2"),
        ]);
        let first = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Best, None);
        for _ in 0..100 {
            let next = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Best, None);
            assert_eq!(first.selected[0].node_id, next.selected[0].node_id);
            assert_eq!(first.confidence, next.confidence);
        }
    }

    #[test]
    fn test_select_best_id() {
        let g = make_graph(vec![
            ("a", 5, 0, 0, "1"),
            ("b", 8, 1, 1, "1"),
        ]);
        let id = ObjectSelector::select_best_id(&LargestPredicate, &g);
        assert_eq!(id, Some("b".to_string()));
    }
}
