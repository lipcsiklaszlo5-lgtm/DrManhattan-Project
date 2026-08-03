use crate::sandbox::operators::Transformation;
use crate::structure::KernelStructureGraph;
use crate::structure::topology::{graph_diff, NodeTransformation};
use super::transform::{TransformRule, TransformationAlgebra, Condition};
use crate::concept::{Concept, ConceptRegistry};
use crate::predicate::{Predicate, PredicateResult};
use crate::predicate::builtin::{ColorPredicate, LargestPredicate, OnlyObjectPredicate, AndPredicate};

// --- Program ---
#[derive(Debug, Clone)]
pub struct Program {
    pub steps: Vec<Transformation>,
    pub confidence: f32,
    pub success_count: u64,
}

impl Program {
    pub fn new(steps: Vec<Transformation>) -> Self { Self { steps, confidence: 0.5, success_count: 1 } }
    pub fn cost(&self) -> f64 { (1.0 - self.confidence as f64) * (self.steps.len() as f64).max(1.0) }
    pub fn apply(&self, graph: &KernelStructureGraph) -> KernelStructureGraph {
        let mut current = graph.clone();
        for step in &self.steps { current = crate::sandbox::operators::apply_transformation(&current, step); }
        current
    }
    pub fn record_success(&mut self) { self.success_count += 1; self.confidence = (self.confidence + 0.1).min(1.0); }
    pub fn record_failure(&mut self) { self.confidence = (self.confidence - 0.1).max(0.0); }
}

// --- GeneralizedProgram ---
#[derive(Debug, Clone)]
pub struct GeneralizedProgram {
    pub steps: Vec<AbstractStep>,
    pub confidence: f32,
    pub num_train_pairs: usize,
}

pub struct AbstractStep {
    pub condition: Option<Box<dyn Predicate>>,
    pub transformation: Transformation,
    pub target_spec: Option<TargetSpec>,
    pub cardinality: Cardinality,
}

impl Clone for AbstractStep {
    fn clone(&self) -> Self {
        AbstractStep {
            condition: self.condition.as_ref().map(|c| c.clone_box()),
            transformation: self.transformation.clone(),
            target_spec: self.target_spec.clone(),
            cardinality: self.cardinality.clone(),
        }
    }
}

impl std::fmt::Debug for AbstractStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AbstractStep")
            .field("condition", &self.condition.as_ref().map(|c| c.name()))
            .field("transformation", &self.transformation)
            .field("target_spec", &self.target_spec)
            .field("cardinality", &self.cardinality)
            .finish()
    }
}

pub enum TargetSpec {
    Constant(String),
    RelativeToNode { condition: Box<Condition>, dx_offset: i64, dy_offset: i64 },
    GridAnchor { corner: GridCorner },
    CopyAttributeFrom { condition: Box<Condition>, attribute: String },
    GravitateAnchor { anchor_predicate: Box<dyn Predicate> },
}


impl PartialEq for TargetSpec {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TargetSpec::Constant(a), TargetSpec::Constant(b)) => a == b,
            (TargetSpec::RelativeToNode { condition: c1, dx_offset: dx1, dy_offset: dy1 },
             TargetSpec::RelativeToNode { condition: c2, dx_offset: dx2, dy_offset: dy2 }) =>
                c1 == c2 && dx1 == dx2 && dy1 == dy2,
            (TargetSpec::GridAnchor { corner: c1 }, TargetSpec::GridAnchor { corner: c2 }) => c1 == c2,
            (TargetSpec::CopyAttributeFrom { condition: c1, attribute: a1 },
             TargetSpec::CopyAttributeFrom { condition: c2, attribute: a2 }) => c1 == c2 && a1 == a2,
            (TargetSpec::GravitateAnchor { anchor_predicate: p1 },
             TargetSpec::GravitateAnchor { anchor_predicate: p2 }) => p1.name() == p2.name(),
            _ => false,
        }
    }
}

impl std::fmt::Debug for TargetSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetSpec::Constant(v) => write!(f, "Constant({})", v),
            TargetSpec::RelativeToNode { condition, dx_offset, dy_offset } =>
                write!(f, "RelativeToNode({}, {}, {})", condition.name(), dx_offset, dy_offset),
            TargetSpec::GridAnchor { corner } => write!(f, "GridAnchor({:?})", corner),
            TargetSpec::CopyAttributeFrom { condition, attribute } =>
                write!(f, "CopyAttributeFrom({}, {})", condition.name(), attribute),
            TargetSpec::GravitateAnchor { anchor_predicate } =>
                write!(f, "GravitateAnchor({})", anchor_predicate.name()),
        }
    }
}

impl Clone for TargetSpec {
    fn clone(&self) -> Self {
        match self {
            TargetSpec::Constant(v) => TargetSpec::Constant(v.clone()),
            TargetSpec::RelativeToNode { condition, dx_offset, dy_offset } =>
                TargetSpec::RelativeToNode { condition: condition.clone(), dx_offset: *dx_offset, dy_offset: *dy_offset },
            TargetSpec::GridAnchor { corner } => TargetSpec::GridAnchor { corner: corner.clone() },
            TargetSpec::CopyAttributeFrom { condition, attribute } =>
                TargetSpec::CopyAttributeFrom { condition: condition.clone(), attribute: attribute.clone() },
            TargetSpec::GravitateAnchor { anchor_predicate } =>
                TargetSpec::GravitateAnchor { anchor_predicate: anchor_predicate.clone_box() },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GridCorner { TopLeft, TopRight, BottomLeft, BottomRight }

#[derive(Debug, Clone, PartialEq)]
pub enum Cardinality { All, ExactlyOne, AtMostOne }

impl GeneralizedProgram {
    pub fn new(steps: Vec<AbstractStep>, confidence: f32, num_train_pairs: usize) -> Self {
        Self { steps, confidence, num_train_pairs }
    }

    /// Kompatibilitasi adapter: a regi "matching_nodes -> foreach" hivasi
    /// mintat tartja eletben, de belulről mar az ObjectSelector::select-et
    /// hasznalja SelectionStrategy::All strategiaval, hogy egyetlen kozos
    /// utvonalon menjen at minden jelolt-kivalasztas.
    pub fn matching_nodes<'a>(graph: &'a KernelStructureGraph, predicate: &dyn Predicate) -> Vec<&'a crate::structure::Node> {
        let result = crate::object_selector::ObjectSelector::select(
            predicate,
            graph,
            &crate::object_selector::SelectionStrategy::All,
            None,
        );
        result.selected.iter()
            .filter_map(|sel| graph.nodes.iter().find(|n| n.id == sel.node_id))
            .collect()
    }

    /// Cardinality -> SelectionStrategy forditas.
    fn strategy_for_cardinality(cardinality: &Cardinality) -> crate::object_selector::SelectionStrategy {
        match cardinality {
            Cardinality::All => crate::object_selector::SelectionStrategy::All,
            Cardinality::ExactlyOne => crate::object_selector::SelectionStrategy::Unique,
            Cardinality::AtMostOne => crate::object_selector::SelectionStrategy::Best,
        }
    }

    /// Jeloltek beszerzese az ObjectSelector-on keresztul, a lepes
    /// Cardinality mezoje altal meghatarozott strategiaval. Ha nincs
    /// condition, minden node jelolt marad (visszafele kompatibilis a
    /// korabbi "None -> osszes node" viselkedessel).
    fn select_candidates<'a>(
        graph: &'a KernelStructureGraph,
        step: &AbstractStep,
    ) -> Option<Vec<&'a crate::structure::Node>> {
        match &step.condition {
            None => Some(graph.nodes.iter().collect()),
            Some(pred) => {
                let strategy = Self::strategy_for_cardinality(&step.cardinality);
                let result = crate::object_selector::ObjectSelector::select(
                    pred.as_ref(), graph, &strategy, None,
                );

                // Cardinality::ExactlyOne eseten, ha a valasztas ketertelmu
                // vagy nincs talalat, NEM talalgatunk -- a lepes ezen a
                // bemeneten nem hajt vegre semmit, inkabb mint hogy hibas
                // node-ot valasszon.
                if matches!(step.cardinality, Cardinality::ExactlyOne) {
                    if result.ambiguity || result.selected.is_empty() {
                        return None;
                    }
                }

                let nodes: Vec<&crate::structure::Node> = result.selected.iter()
                    .filter_map(|sel| graph.nodes.iter().find(|n| n.id == sel.node_id))
                    .collect();
                Some(nodes)
            }
        }
    }

    fn resolve_target_spec(
        spec: &TargetSpec,
        graph: &KernelStructureGraph,
        grid_width: u8,
        grid_height: u8,
    ) -> Option<(i64, i64, Option<String>)> {
        match spec {
            TargetSpec::Constant(val) => {
                if let Ok(v) = val.parse::<i64>() { Some((v, 0, Some(val.clone()))) }
                else { Some((0, 0, Some(val.clone()))) }
            }
            TargetSpec::RelativeToNode { condition, dx_offset, dy_offset } => {
                let refs = Self::matching_nodes(graph, condition.as_ref());
                if let Some(ref_node) = refs.first() {
                    let rx: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let ry: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                    Some((rx + dx_offset, ry + dy_offset, None))
                } else { None }
            }
            TargetSpec::GridAnchor { corner } => {
                let (tx, ty) = match corner {
                    GridCorner::TopLeft => (0i64, 0i64),
                    GridCorner::TopRight => (grid_width as i64, 0),
                    GridCorner::BottomLeft => (0, grid_height as i64),
                    GridCorner::BottomRight => (grid_width as i64, grid_height as i64),
                };
                Some((tx, ty, None))
            }
            TargetSpec::CopyAttributeFrom { condition, attribute } => {
                let refs = Self::matching_nodes(graph, condition.as_ref());
                if let Some(ref_node) = refs.first() {
                    let val = ref_node.attributes.get(attribute).cloned();
                    Some((0, 0, val))
                } else { None }
            }
            TargetSpec::GravitateAnchor { anchor_predicate } => {
                let refs = Self::matching_nodes(graph, anchor_predicate.as_ref());
                if let Some(ref_node) = refs.first() {
                    let rx: i64 = ref_node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let ry: i64 = ref_node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                    Some((rx, ry, None))
                } else { None }
            }

        }
    }

    fn apply_step(graph: &KernelStructureGraph, step: &AbstractStep, gw: u8, gh: u8) -> KernelStructureGraph {
        let candidates: Vec<crate::structure::Node> = match Self::select_candidates(graph, step) {
            Some(nodes) => nodes.into_iter().cloned().collect(),
            None => Vec::new(), // Cardinality::ExactlyOne, de ambiguity/0 talalat -> nincs vegrehajtas
        };

        let mut result = graph.clone();
        for node in candidates {
            let mut transformation = step.transformation.clone();

            if let Some(ref spec) = step.target_spec {
                if let Some((tx, ty, color_opt)) = Self::resolve_target_spec(spec, graph, gw, gh) {
                    match &mut transformation {
                        Transformation::Translate { dx, dy, .. } => {
                            let ox: i64 = node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                            let oy: i64 = node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                            *dx = tx - ox;
                            *dy = ty - oy;
                        }
                        Transformation::TranslateToTarget { .. } => {
                            let ox: i64 = node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                            let oy: i64 = node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                            transformation = Transformation::Translate { node_id: node.id.clone(), dx: tx - ox, dy: ty - oy };
                        }
                        Transformation::RecolorToTarget { .. } | Transformation::Recolor { .. } => {
                            if let Some(color) = color_opt {
                                transformation = Transformation::Recolor { node_id: node.id.clone(), new_color: color };
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Handle semantic mirror transformations here because they need grid dimensions
            match &transformation {
                Transformation::MirrorHorizontal { node_id } => {
                    result = crate::sandbox::operators::apply_mirror_horizontal(&result, node_id, gw);
                }
                Transformation::MirrorVertical { node_id } => {
                    result = crate::sandbox::operators::apply_mirror_vertical(&result, node_id, gh);
                }
                Transformation::SemanticMirrorHorizontal => {
                    result = crate::sandbox::operators::apply_mirror_horizontal(&result, &node.id, gw);
                }
                Transformation::SemanticMirrorVertical => {
                    result = crate::sandbox::operators::apply_mirror_vertical(&result, &node.id, gh);
                }
                Transformation::SemanticTranslateToTarget => {
                    // Use target_spec to compute translation, then apply Translate with computed dx,dy
                    if let Some(spec) = &step.target_spec {
                        if let Some((tx, ty, color_opt)) = Self::resolve_target_spec(spec, graph, gw, gh) {
                            let ox: i64 = node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
                            let oy: i64 = node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
                            let dx = tx - ox;
                            let dy = ty - oy;
                            let translate = Transformation::Translate { node_id: node.id.clone(), dx, dy };
                            result = crate::sandbox::operators::apply_transformation(&result, &translate);
                        }
                    }
                }
                Transformation::SemanticRecolorToTarget => {
                    if let Some(spec) = &step.target_spec {
                        if let Some((_, _, Some(color))) = Self::resolve_target_spec(spec, graph, gw, gh) {
                            let recolor = Transformation::Recolor { node_id: node.id.clone(), new_color: color };
                            result = crate::sandbox::operators::apply_transformation(&result, &recolor);
                        }
                    }
                }
                _ => {
                    result = crate::sandbox::operators::apply_transformation(&result, &transformation);
                }
            }
        }
        result
    }

    pub fn apply(&self, graph: &KernelStructureGraph, gw: u8, gh: u8) -> KernelStructureGraph {
        let mut current = graph.clone();
        for step in &self.steps {
            current = Self::apply_step(&current, step, gw, gh);
        }
        current
    }
}

// --- ProgramSynthesizer (optimalizált predikátum-cache-sel) ---
pub struct ProgramSynthesizer {
    pub programs: Vec<Program>,
    pub generalized_programs: Vec<GeneralizedProgram>,
    pub algebra: TransformationAlgebra,
    concept_transforms: Vec<(Concept, String)>,
    pub concept_registry: ConceptRegistry,
    /// Predikátum kiértékelések cache: (graph_fingerprint, predicate_name) -> PredicateResult
    predicate_cache: HashMap<(u64, String), PredicateResult>,
}

impl ProgramSynthesizer {
    pub fn new() -> Self {
        let mut ct = Vec::new();
        ct.push((Concept::Connected, "Translate".into()));
        ct.push((Concept::Symmetry, "Recolor".into()));
        ct.push((Concept::Player, "Translate".into()));
        ct.push((Concept::Exit, "Translate".into()));
        ct.push((Concept::Hole, "Delete".into()));
        ct.push((Concept::Largest, "Translate".into()));

        Self {
            programs: Vec::new(),
            generalized_programs: Vec::new(),
            algebra: TransformationAlgebra::new(),
            concept_transforms: ct,
            concept_registry: ConceptRegistry::default(),
            predicate_cache: HashMap::new(),
        }
    }

    pub fn map_concept_to_transform(&mut self, concept: Concept, transform_name: &str) {
        self.concept_transforms.push((concept, transform_name.to_string()));
    }

    pub fn transforms_for_concept(&self, concept: &Concept) -> Vec<String> {
        self.concept_transforms.iter().filter(|(c, _)| c == concept).map(|(_, t)| t.clone()).collect()
    }

    pub fn learn_from_example(&mut self, before: &KernelStructureGraph, after: &KernelStructureGraph) -> Option<Program> {
        let diffs: Vec<_> = graph_diff(before, after).into_iter()
            .filter(|d| !matches!(d, NodeTransformation::Unchanged { .. })).collect();
        if diffs.is_empty() { return None; }

        let mut steps = Vec::new();
        for diff in &diffs {
            if let Some(step) = Self::diff_to_transformation(diff, after) {
                steps.push(step);
            }
        }
        if steps.is_empty() { return None; }

        let program = Program::new(steps);
        self.programs.push(program.clone());
        for step in &program.steps { self.algebra.add_rule(TransformRule::new(step.clone())); }
        Some(program)
    }

    /// Optimalizált tanulás – a predikátum cache-t használja
    pub fn learn_generalized(&mut self, before: &KernelStructureGraph, after: &KernelStructureGraph, grid_width: u8, grid_height: u8) -> Option<GeneralizedProgram> {
        // Use Semantic Hypothesis Engine to generate pure steps
        let semantic_steps = crate::semantic_hypothesis::generator::generate_candidate_steps(before, after, grid_width, grid_height);
        if semantic_steps.is_empty() {
            return None;
        }

        let steps: Vec<AbstractStep> = semantic_steps.into_iter().map(|s| {
            AbstractStep {
                condition: s.condition.map(|preds| {
                    if preds.len() == 1 {
                        preds[0].clone_box()
                    } else {
                        Box::new(AndPredicate { predicates: preds.iter().map(|p| p.clone_box()).collect() })
                    }
                }),
                transformation: s.transformation,
                target_spec: s.target_spec,
                cardinality: Cardinality::All, // default, will be refined by later modules
            }
        }).collect();

        if steps.is_empty() {
            return None;
        }

        let program = GeneralizedProgram::new(steps, 0.8, 1);
        self.generalized_programs.push(program.clone());
        Some(program)
    }

    fn get_node_id(diff: &NodeTransformation) -> Option<String> {
        match diff {
            NodeTransformation::Translate { node_id, .. } => Some(node_id.clone()),
            NodeTransformation::Recolor { node_id, .. } => Some(node_id.clone()),
            NodeTransformation::Delete { node_id } => Some(node_id.clone()),
            NodeTransformation::Rotate { node_id, .. } => Some(node_id.clone()),
            _ => None,
        }
    }

    fn diff_to_transformation(diff: &NodeTransformation, after: &KernelStructureGraph) -> Option<Transformation> {
        match diff {
            NodeTransformation::Translate { node_id, dx, dy } => Some(Transformation::Translate { node_id: node_id.clone(), dx: *dx, dy: *dy }),
            NodeTransformation::Recolor { node_id, new_color } => Some(Transformation::Recolor { node_id: node_id.clone(), new_color: new_color.clone() }),
            NodeTransformation::Delete { node_id } => Some(Transformation::Delete { node_id: node_id.clone() }),
            NodeTransformation::Create { node_id: nid, color, bbox_x, bbox_y } => {
                if let Some(new_node) = after.nodes.iter().find(|n| n.id == *nid) {
                    let bw = new_node.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1);
                    let bh = new_node.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1);
                    Some(Transformation::Create { color: color.clone(), bbox_x: *bbox_x, bbox_y: *bbox_y, bbox_w: bw, bbox_h: bh })
                } else { None }
            }
            NodeTransformation::Rotate { node_id, angle } => Some(Transformation::Rotate { node_id: node_id.clone(), angle: *angle }),
            NodeTransformation::Unchanged { .. } => None,
        }
    }

    fn find_best_predicate_cached(
        available: &[Box<dyn Predicate>],
        graph: &KernelStructureGraph,
        affected_ids: &[String],
        cache: &mut HashMap<(u64, String), PredicateResult>,
    ) -> Option<Box<dyn Predicate>> {
        let fp = graph.fingerprint();
        let mut best: Option<Box<dyn Predicate>> = None;
        let mut best_score: f32 = 0.0;

        for predicate in available {
            let cache_key = (fp, predicate.name().to_string());
            let result = if let Some(cached) = cache.get(&cache_key) {
                cached.clone()
            } else {
                let result = predicate.evaluate(graph);
                cache.insert(cache_key, result.clone());
                result
            };

            if let PredicateResult::RankedList(matching) = result {
                let matching_ids: Vec<&str> = matching.iter().map(|(id, _)| id.as_str()).collect();
                let affected_count = affected_ids.iter().filter(|id| matching_ids.contains(&id.as_str())).count();
                let precision = if matching_ids.is_empty() { 0.0 } else { affected_count as f32 / matching_ids.len() as f32 };
                let recall = if affected_ids.is_empty() { 0.0 } else { affected_count as f32 / affected_ids.len() as f32 };
                let score = 2.0 * precision * recall / (precision + recall + 0.001);
                if score > best_score { best_score = score; best = Some(predicate.clone_box()); }
            }
        }
        best
    }

    pub fn generalize_from_pairs(&mut self, _pairs: &[(KernelStructureGraph, KernelStructureGraph)]) -> Option<GeneralizedProgram> { None }
    pub fn find_best_program(&self, graph: &KernelStructureGraph, target: &KernelStructureGraph) -> Option<Program> {
        self.programs.iter().filter(|p| { let r = p.apply(graph); r.nodes.len() == target.nodes.len() })
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap()).cloned()
    }
    pub fn find_best_generalized(&self) -> Option<&GeneralizedProgram> {
        self.generalized_programs.iter().max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
    }
    pub fn consolidate(&mut self) {
        self.programs.retain(|p| p.confidence > 0.4);
        self.generalized_programs.retain(|g| g.confidence > 0.4);
        // Purge impure programs (those containing concrete node IDs)
        self.generalized_programs.retain(|g| Self::is_program_pure(g));
        self.concept_registry.consolidate();
        if self.predicate_cache.len() > 1000 {
            self.predicate_cache.clear();
        }
        if self.programs.len() > 50 {
            self.programs.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
            self.programs.truncate(50);
        }
    }

    fn is_program_pure(prog: &GeneralizedProgram) -> bool {
        for step in &prog.steps {
            if let Some(cond) = &step.condition {
                let name = cond.name();
                if name.contains("obj_") || name.contains("bbox_x") || name.contains("bbox_y") {
                    return false;
                }
            }
            match &step.transformation {
                Transformation::Translate { node_id, .. } |
                Transformation::Recolor { node_id, .. } |
                Transformation::Delete { node_id } |
                Transformation::Rotate { node_id, .. } => {
                    if node_id.contains("obj_") {
                        return false;
                    }
                }
                Transformation::Create { bbox_x: _, bbox_y: _, .. } => {
                    return false; // absolute coords forbidden
                }
                _ => {}
            }
        }
        true
    }
}

use std::collections::HashMap;
