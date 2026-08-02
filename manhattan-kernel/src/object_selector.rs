use crate::predicate::{Predicate, PredicateResult};
use crate::structure::KernelStructureGraph;
use std::collections::HashMap;
use std::cmp::Ordering;

// =====================================================================
// TieBreaker trait – independent abstraction for deterministic tie resolution
// =====================================================================
pub trait TieBreaker: Send + Sync {
    fn compare(&self, a: &SelectedObject, b: &SelectedObject) -> Ordering;
    fn is_semantic_tie(&self, a: &SelectedObject, b: &SelectedObject) -> bool;
}

fn numeric_suffix(id: &str) -> Option<u64> {
    let digits: String = id.chars().rev().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() { return None; }
    let digits: String = digits.chars().rev().collect();
    digits.parse::<u64>().ok()
}

pub struct DefaultTieBreaker;

impl TieBreaker for DefaultTieBreaker {
    fn compare(&self, a: &SelectedObject, b: &SelectedObject) -> Ordering {
        b.score
            .partial_cmp(&a.score).unwrap_or(Ordering::Equal)
            .then_with(|| b.predicate_confidence.partial_cmp(&a.predicate_confidence).unwrap_or(Ordering::Equal))
            .then_with(|| b.specificity.cmp(&a.specificity))
            .then_with(|| match (numeric_suffix(&a.node_id), numeric_suffix(&b.node_id)) {
                (Some(x), Some(y)) => x.cmp(&y),
                _ => a.node_id.cmp(&b.node_id),
            })
    }

    fn is_semantic_tie(&self, a: &SelectedObject, b: &SelectedObject) -> bool {
        a.score == b.score
            && a.predicate_confidence == b.predicate_confidence
            && a.specificity == b.specificity
    }
}

// =====================================================================
// RankingEngine trait
// =====================================================================
pub trait RankingEngine: Send + Sync {
    fn rank(&self, candidates: Vec<SelectedObject>, tie_breaker: &dyn TieBreaker) -> Vec<SelectedObject>;
}

pub struct DefaultRankingEngine;

impl RankingEngine for DefaultRankingEngine {
    fn rank(&self, mut candidates: Vec<SelectedObject>, tie_breaker: &dyn TieBreaker) -> Vec<SelectedObject> {
        candidates.sort_by(|a, b| tie_breaker.compare(a, b));
        candidates
    }
}

// =====================================================================
// Selection strategies
// =====================================================================
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionStrategy {
    Best,
    TopK(usize),
    Threshold(f32),
    All,
    Unique,
}

// =====================================================================
// Scoring
// =====================================================================
pub type ScoreFn = fn(&crate::structure::Node, f32) -> f32;

#[derive(Debug, Clone)]
pub struct SelectedObject {
    pub node_id: String,
    pub score: f32,
    pub predicate_confidence: f32,
    pub specificity: u32,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct SelectionResult {
    pub selected: Vec<SelectedObject>,
    pub ranking: Vec<SelectedObject>,
    pub ambiguity: bool,
    pub confidence: f32,
    pub explanation: String,
}

impl SelectionResult {
    pub fn best_id(&self) -> Option<&str> {
        self.selected.first().map(|s| s.node_id.as_str())
    }
    pub fn is_unique(&self) -> bool {
        self.selected.len() == 1 && !self.ambiguity
    }
}

fn build_explanation(obj: &SelectedObject, ambiguity: bool) -> String {
    if ambiguity {
        format!(
            "Selected: {} | Reason: {} | Score: {:.3} (tobb jelolt holtversenyben allt, determinisztikusan feloldva)",
            obj.node_id, obj.reason, obj.score
        )
    } else {
        format!(
            "Selected: {} | Reason: {} | Score: {:.3}",
            obj.node_id, obj.reason, obj.score
        )
    }
}

// ---------------------------------------------------------------------
// Extensible scoring system
// ---------------------------------------------------------------------
pub trait ScoringComponent: Send + Sync {
    fn name(&self) -> &str;
    fn weight(&self) -> f32 { 1.0 }
    fn score(&self, node: &crate::structure::Node, predicate_confidence: f32, graph: &KernelStructureGraph) -> f32;
}

pub struct PredicateConfidenceComponent { pub weight: f32 }
impl Default for PredicateConfidenceComponent {
    fn default() -> Self { Self { weight: 1.0 } }
}
impl ScoringComponent for PredicateConfidenceComponent {
    fn name(&self) -> &str { "PredicateConfidence" }
    fn weight(&self) -> f32 { self.weight }
    fn score(&self, _node: &crate::structure::Node, predicate_confidence: f32, _graph: &KernelStructureGraph) -> f32 {
        predicate_confidence
    }
}

pub struct AreaComponent { pub weight: f32 }
impl Default for AreaComponent {
    fn default() -> Self { Self { weight: 1.0 } }
}
impl ScoringComponent for AreaComponent {
    fn name(&self) -> &str { "Area" }
    fn weight(&self) -> f32 { self.weight }
    fn score(&self, node: &crate::structure::Node, _predicate_confidence: f32, _graph: &KernelStructureGraph) -> f32 {
        let area: f32 = node.attributes.get("area").and_then(|v| v.parse().ok()).unwrap_or(1.0);
        area.log10().max(0.0)
    }
}

pub struct PositionComponent { pub weight: f32, pub grid_width: f32, pub grid_height: f32 }
impl PositionComponent {
    pub fn new(weight: f32, grid_width: f32, grid_height: f32) -> Self {
        Self { weight, grid_width, grid_height }
    }
}
impl ScoringComponent for PositionComponent {
    fn name(&self) -> &str { "Position" }
    fn weight(&self) -> f32 { self.weight }
    fn score(&self, node: &crate::structure::Node, _predicate_confidence: f32, _graph: &KernelStructureGraph) -> f32 {
        let x: f32 = node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let y: f32 = node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let cx = self.grid_width / 2.0;
        let cy = self.grid_height / 2.0;
        let dist = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
        let max_dist = (cx.powi(2) + cy.powi(2)).sqrt().max(1.0);
        1.0 - (dist / max_dist).min(1.0)
    }
}

pub struct TopologyComponent { pub weight: f32 }
impl Default for TopologyComponent {
    fn default() -> Self { Self { weight: 1.0 } }
}
impl ScoringComponent for TopologyComponent {
    fn name(&self) -> &str { "Topology" }
    fn weight(&self) -> f32 { self.weight }
    fn score(&self, node: &crate::structure::Node, _predicate_confidence: f32, graph: &KernelStructureGraph) -> f32 {
        let degree = graph.edges.iter()
            .filter(|e| e.from == node.id || e.to == node.id)
            .count();
        (degree as f32).log10().max(0.0)
            .max(if degree > 0 { 0.1 } else { 0.0 })
    }
}

pub struct SymmetryComponent { pub weight: f32 }
impl Default for SymmetryComponent {
    fn default() -> Self { Self { weight: 1.0 } }
}
impl ScoringComponent for SymmetryComponent {
    fn name(&self) -> &str { "Symmetry" }
    fn weight(&self) -> f32 { self.weight }
    fn score(&self, node: &crate::structure::Node, _predicate_confidence: f32, _graph: &KernelStructureGraph) -> f32 {
        if node.attributes.contains_key("symmetry") { 1.0 } else { 0.0 }
    }
}

pub struct ConceptConfidenceComponent { pub weight: f32 }
impl Default for ConceptConfidenceComponent {
    fn default() -> Self { Self { weight: 1.0 } }
}
impl ScoringComponent for ConceptConfidenceComponent {
    fn name(&self) -> &str { "ConceptConfidence" }
    fn weight(&self) -> f32 { self.weight }
    fn score(&self, node: &crate::structure::Node, _predicate_confidence: f32, _graph: &KernelStructureGraph) -> f32 {
        node.attributes.get("concept_confidence").and_then(|v| v.parse().ok()).unwrap_or(0.0)
    }
}

struct FnComponent(ScoreFn);
impl ScoringComponent for FnComponent {
    fn name(&self) -> &str { "LegacyScoreFn" }
    fn score(&self, node: &crate::structure::Node, predicate_confidence: f32, _graph: &KernelStructureGraph) -> f32 {
        (self.0)(node, predicate_confidence)
    }
}

pub struct ScoringProfile {
    pub components: Vec<Box<dyn ScoringComponent>>,
}

impl ScoringProfile {
    pub fn new(components: Vec<Box<dyn ScoringComponent>>) -> Self {
        Self { components }
    }

    pub fn add_component(&mut self, component: Box<dyn ScoringComponent>) {
        self.components.push(component);
    }

    pub fn default_profile() -> Self {
        Self::new(vec![
            Box::new(PredicateConfidenceComponent::default()),
            Box::new(AreaComponent::default()),
        ])
    }

    fn compute(&self, node: &crate::structure::Node, predicate_confidence: f32, graph: &KernelStructureGraph) -> f32 {
        if self.components.is_empty() {
            return predicate_confidence;
        }
        if self.components.len() == 2
            && self.components[0].name() == "PredicateConfidence"
            && self.components[1].name() == "Area"
        {
            let pred = self.components[0].score(node, predicate_confidence, graph);
            let area = self.components[1].score(node, predicate_confidence, graph);
            return pred * (1.0 + area);
        }
        self.components.iter()
            .map(|c| c.weight() * c.score(node, predicate_confidence, graph))
            .sum()
    }
}

// =====================================================================
// ObjectSelector
// =====================================================================
pub struct ObjectSelector;

impl ObjectSelector {
    pub fn default_score_fn(node: &crate::structure::Node, predicate_score: f32) -> f32 {
        let area: f32 = node.attributes.get("area").and_then(|v| v.parse().ok()).unwrap_or(1.0);
        predicate_score * (1.0 + area.log10().max(0.0))
    }

    pub fn select(
        predicate: &dyn Predicate,
        graph: &KernelStructureGraph,
        strategy: &SelectionStrategy,
        score_fn: Option<ScoreFn>,
    ) -> SelectionResult {
        let profile = match score_fn {
            Some(f) => ScoringProfile::new(vec![Box::new(FnComponent(f))]),
            None => ScoringProfile::default_profile(),
        };
        Self::select_with_scoring(predicate, graph, strategy, &profile)
    }

    pub fn select_with_scoring(
        predicate: &dyn Predicate,
        graph: &KernelStructureGraph,
        strategy: &SelectionStrategy,
        profile: &ScoringProfile,
    ) -> SelectionResult {
        let tie_breaker = DefaultTieBreaker;
        let ranking_engine = DefaultRankingEngine;
        let pred_specificity = predicate.specificity();
        let pred_name = predicate.name().to_string();

        // 1. Predicate evaluation
        let candidates = match predicate.evaluate(graph) {
            PredicateResult::RankedList(list) => list,
            PredicateResult::Bool(true) => graph.nodes.iter().map(|n| (n.id.clone(), 1.0)).collect(),
            PredicateResult::Bool(false) => Vec::new(),
        };

        if candidates.is_empty() {
            return SelectionResult {
                selected: Vec::new(),
                ranking: Vec::new(),
                ambiguity: false,
                confidence: 0.0,
                explanation: "Nem talalhato jelolt objektum.".to_string(),
            };
        }

        // 2. Scoring
        let node_map: HashMap<&str, &crate::structure::Node> = graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
        let scored: Vec<SelectedObject> = candidates
            .into_iter()
            .filter_map(|(id, pred_score)| {
                node_map.get(id.as_str()).map(|&node| {
                    let score = profile.compute(node, pred_score, graph);
                    SelectedObject {
                        node_id: id.clone(),
                        score,
                        predicate_confidence: pred_score,
                        specificity: pred_specificity,
                        reason: pred_name.clone(),
                    }
                })
            })
            .collect();

        // 3. Ranking
        let ranking = ranking_engine.rank(scored, &tie_breaker);

        // 4. Strategy application
        Self::apply_strategy(strategy, ranking, &tie_breaker)
    }

    fn apply_strategy(
        strategy: &SelectionStrategy,
        ranking: Vec<SelectedObject>,
        tie_breaker: &DefaultTieBreaker,
    ) -> SelectionResult {
        match strategy {
            SelectionStrategy::Best => Self::apply_best(ranking, tie_breaker),
            SelectionStrategy::TopK(k) => Self::apply_topk(ranking, *k),
            SelectionStrategy::Threshold(t) => Self::apply_threshold(ranking, *t),
            SelectionStrategy::All => Self::apply_all(ranking),
            SelectionStrategy::Unique => Self::apply_unique(ranking, tie_breaker),
        }
    }

    fn apply_best(ranking: Vec<SelectedObject>, tie_breaker: &DefaultTieBreaker) -> SelectionResult {
        if let Some(top) = ranking.first().cloned() {
            let tie_count = ranking.iter().take_while(|s| tie_breaker.is_semantic_tie(s, &top)).count();
            let ambiguity = tie_count > 1;
            let confidence = top.score;
            let explanation = build_explanation(&top, ambiguity);
            SelectionResult { selected: vec![top], ranking, ambiguity, confidence, explanation }
        } else {
            SelectionResult { selected: vec![], ranking, ambiguity: false, confidence: 0.0, explanation: "Nem talalhato jelolt objektum.".to_string() }
        }
    }

    fn apply_topk(ranking: Vec<SelectedObject>, k: usize) -> SelectionResult {
        let selected: Vec<_> = ranking.iter().take(k).cloned().collect();
        let confidence = selected.first().map(|s| s.score).unwrap_or(0.0);
        let explanation = format!("TopK({}) strategia: {} objektum kivalasztva.", k, selected.len());
        SelectionResult { selected, ranking, ambiguity: false, confidence, explanation }
    }

    fn apply_threshold(ranking: Vec<SelectedObject>, t: f32) -> SelectionResult {
        let selected: Vec<_> = ranking.iter().filter(|s| s.score >= t).cloned().collect();
        let confidence = selected.first().map(|s| s.score).unwrap_or(0.0);
        let explanation = format!("Threshold({:.3}) strategia: {} objektum erte el a kuszoboket.", t, selected.len());
        SelectionResult { selected, ranking, ambiguity: false, confidence, explanation }
    }

    fn apply_all(ranking: Vec<SelectedObject>) -> SelectionResult {
        let confidence = ranking.first().map(|s| s.score).unwrap_or(0.0);
        let explanation = format!("All strategia: {} objektum kivalasztva.", ranking.len());
        SelectionResult { selected: ranking.clone(), ranking, ambiguity: false, confidence, explanation }
    }

    fn apply_unique(ranking: Vec<SelectedObject>, _tie_breaker: &DefaultTieBreaker) -> SelectionResult {
        if ranking.len() == 1 {
            let obj = ranking[0].clone();
            let confidence = obj.score;
            let explanation = build_explanation(&obj, false);
            SelectionResult { selected: vec![obj], ranking, ambiguity: false, confidence, explanation }
        } else {
            let top = ranking[0].clone();
            let confidence = top.score;
            let explanation = format!(
                "Ambiguous: {} jelolt felelt meg a feltetelnek (pl. {}). Ok: {}",
                ranking.len(), top.node_id, top.reason
            );
            SelectionResult { selected: vec![top], ranking, ambiguity: true, confidence, explanation }
        }
    }

    pub fn select_best_id(predicate: &dyn Predicate, graph: &KernelStructureGraph) -> Option<String> {
        let result = Self::select(predicate, graph, &SelectionStrategy::Best, None);
        result.best_id().map(|s| s.to_string())
    }
}

// =====================================================================
// Tests
// =====================================================================
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
        let g = make_graph(vec![("a", 5, 0, 0, "1"), ("b", 8, 1, 1, "1"), ("c", 3, 2, 2, "2")]);
        let pred = LargestPredicate;
        let result = ObjectSelector::select(&pred, &g, &SelectionStrategy::Best, None);
        assert_eq!(result.selected.len(), 1);
        assert_eq!(result.selected[0].node_id, "b");
        assert!(!result.ambiguity);
    }

    #[test]
    fn test_topk_strategy() {
        let g = make_graph(vec![("a", 5, 0, 0, "1"), ("b", 8, 1, 1, "1"), ("c", 3, 2, 2, "2"), ("d", 7, 3, 3, "3")]);
        let result = ObjectSelector::select(&ColorPredicate { color: "1".into() }, &g, &SelectionStrategy::TopK(2), None);
        assert_eq!(result.selected.len(), 2);
    }

    #[test]
    fn test_threshold_strategy() {
        let g = make_graph(vec![("a", 5, 0, 0, "1"), ("b", 8, 1, 1, "1")]);
        let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Threshold(0.9), None);
        assert!(!result.selected.is_empty());
    }

    #[test]
    fn test_unique_strategy_with_ambiguity() {
        let g = make_graph(vec![("a", 5, 0, 0, "1"), ("b", 5, 1, 1, "2")]);
        let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Unique, None);
        assert!(result.ambiguity);
        assert_eq!(result.selected.len(), 1);
    }

    #[test]
    fn test_determinism() {
        let g = make_graph(vec![("a", 5, 0, 0, "1"), ("b", 8, 1, 1, "1"), ("c", 3, 2, 2, "2")]);
        let first = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Best, None);
        for _ in 0..100 {
            let next = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Best, None);
            assert_eq!(first.selected[0].node_id, next.selected[0].node_id);
            assert_eq!(first.confidence, next.confidence);
        }
    }

    #[test]
    fn test_select_best_id() {
        let g = make_graph(vec![("a", 5, 0, 0, "1"), ("b", 8, 1, 1, "1")]);
        let id = ObjectSelector::select_best_id(&LargestPredicate, &g);
        assert_eq!(id, Some("b".to_string()));
    }

    #[test]
    fn test_ranking_contains_all_candidates_even_when_selected_is_filtered() {
        let g = make_graph(vec![("a", 5, 0, 0, "1"), ("b", 8, 1, 1, "1"), ("c", 3, 2, 2, "1")]);
        let result = ObjectSelector::select(&ColorPredicate { color: "1".into() }, &g, &SelectionStrategy::Best, None);
        assert_eq!(result.selected.len(), 1);
        assert_eq!(result.ranking.len(), 3);
    }

    #[test]
    fn test_numeric_tie_break_on_equal_score() {
        let g = make_graph(vec![("obj_10", 5, 0, 0, "1"), ("obj_2", 5, 1, 1, "1")]);
        let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Best, None);
        assert_eq!(result.selected[0].node_id, "obj_2");
        assert!(result.ambiguity);
    }

    #[test]
    fn test_explanation_is_non_empty() {
        let g = make_graph(vec![("a", 5, 0, 0, "1")]);
        let result = ObjectSelector::select(&LargestPredicate, &g, &SelectionStrategy::Best, None);
        assert!(!result.explanation.is_empty());
    }

    struct FavorColorNineComponent;
    impl ScoringComponent for FavorColorNineComponent {
        fn name(&self) -> &str { "FavorColorNine" }
        fn weight(&self) -> f32 { 10.0 }
        fn score(&self, node: &crate::structure::Node, _pred_conf: f32, _graph: &KernelStructureGraph) -> f32 {
            if node.attributes.get("color").map(|c| c == "9").unwrap_or(false) { 1.0 } else { 0.0 }
        }
    }

    #[test]
    fn test_custom_scoring_component_changes_ranking() {
        let g = make_graph(vec![("a", 20, 0, 0, "1"), ("b", 5, 1, 1, "9")]);
        let default_result = ObjectSelector::select(&AreaPredicate { min: None, max: None }, &g, &SelectionStrategy::Best, None);
        assert_eq!(default_result.selected[0].node_id, "a");
        let custom_profile = ScoringProfile::new(vec![
            Box::new(PredicateConfidenceComponent::default()),
            Box::new(AreaComponent { weight: 0.01 }),
            Box::new(FavorColorNineComponent),
        ]);
        let custom_result = ObjectSelector::select_with_scoring(&AreaPredicate { min: None, max: None }, &g, &SelectionStrategy::Best, &custom_profile);
        assert_eq!(custom_result.selected[0].node_id, "b");
    }

    #[test]
    fn test_scoring_profile_determinism() {
        let g = make_graph(vec![("a", 5, 0, 0, "1"), ("b", 8, 1, 1, "1")]);
        let profile = ScoringProfile::new(vec![
            Box::new(PredicateConfidenceComponent::default()),
            Box::new(AreaComponent::default()),
            Box::new(TopologyComponent::default()),
            Box::new(SymmetryComponent::default()),
            Box::new(ConceptConfidenceComponent::default()),
        ]);
        let first = ObjectSelector::select_with_scoring(&LargestPredicate, &g, &SelectionStrategy::Best, &profile);
        for _ in 0..50 {
            let next = ObjectSelector::select_with_scoring(&LargestPredicate, &g, &SelectionStrategy::Best, &profile);
            assert_eq!(first.selected[0].node_id, next.selected[0].node_id);
            assert_eq!(first.confidence, next.confidence);
        }
    }

    #[test]
    fn test_default_profile_matches_legacy_score_fn_ranking() {
        let g = make_graph(vec![("a", 5, 0, 0, "1"), ("b", 8, 1, 1, "1"), ("c", 3, 2, 2, "1")]);
        let via_default = ObjectSelector::select(&ColorPredicate { color: "1".into() }, &g, &SelectionStrategy::TopK(3), None);
        let via_profile = ObjectSelector::select_with_scoring(
            &ColorPredicate { color: "1".into() }, &g, &SelectionStrategy::TopK(3), &ScoringProfile::default_profile(),
        );
        let ids_a: Vec<&str> = via_default.selected.iter().map(|s| s.node_id.as_str()).collect();
        let ids_b: Vec<&str> = via_profile.selected.iter().map(|s| s.node_id.as_str()).collect();
        assert_eq!(ids_a, ids_b);
    }

    #[test]
    fn test_add_component_method() {
        let mut profile = ScoringProfile::default_profile();
        let initial_count = profile.components.len();
        profile.add_component(Box::new(TopologyComponent::default()));
        assert_eq!(profile.components.len(), initial_count + 1);
        assert_eq!(profile.components.last().unwrap().name(), "Topology");
    }
}
