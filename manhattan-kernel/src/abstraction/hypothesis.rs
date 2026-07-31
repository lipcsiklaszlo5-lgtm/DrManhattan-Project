use crate::structure::KernelStructureGraph;
use crate::adapter::arc::adapter::ArcGrid;
use crate::abstraction::representation::RepresentationFactory;
use crate::abstraction::program::{Program, ProgramSynthesizer};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Hypothesis {
    pub representation_name: String,
    pub representation: KernelStructureGraph,
    pub program: Option<Program>,
    pub confidence: f32,
    pub success_count: u64,
    pub total_attempts: u64,
}

impl Hypothesis {
    pub fn new(name: String, representation: KernelStructureGraph) -> Self {
        Self {
            representation_name: name,
            representation,
            program: None,
            confidence: 0.5,
            success_count: 0,
            total_attempts: 0,
        }
    }

    pub fn score(&self) -> f64 {
        let program_bonus = if self.program.is_some() { 0.5 } else { 0.0 };
        let success_rate = if self.total_attempts > 0 {
            self.success_count as f64 / self.total_attempts as f64
        } else {
            0.5
        };
        (self.confidence as f64 + success_rate) / 2.0 + program_bonus
    }

    pub fn record_success(&mut self) {
        self.success_count += 1;
        self.total_attempts += 1;
        self.confidence = (self.confidence + 0.1).min(1.0);
        if let Some(ref mut program) = self.program {
            program.record_success();
        }
    }

    pub fn record_failure(&mut self) {
        self.total_attempts += 1;
        self.confidence = (self.confidence - 0.05).max(0.0);
        if let Some(ref mut program) = self.program {
            program.record_failure();
        }
    }
}

pub struct HypothesisManager {
    pub hypotheses: Vec<Hypothesis>,
    pub representation_stats: HashMap<String, (u64, u64)>,
    pub cost_weights: HashMap<String, f64>,
}

impl HypothesisManager {
    pub fn new() -> Self {
        let mut cost_weights = HashMap::new();
        cost_weights.insert("Translate".into(), 1.0);
        cost_weights.insert("Recolor".into(), 1.0);
        cost_weights.insert("Delete".into(), 1.5);
        cost_weights.insert("Create".into(), 2.0);
        cost_weights.insert("Merge".into(), 2.5);
        cost_weights.insert("Split".into(), 3.0);

        Self {
            hypotheses: Vec::new(),
            representation_stats: HashMap::new(),
            cost_weights,
        }
    }

    pub fn process_grid(
        &mut self,
        grid: &ArcGrid,
        synthesizer: &mut ProgramSynthesizer,
        target: Option<&KernelStructureGraph>,
    ) {
        let representations = RepresentationFactory::all_representations(grid);
        self.hypotheses.clear();

        for (name, rep) in representations {
            let mut hypothesis = Hypothesis::new(name.clone(), rep.clone());

            if let Some(target_graph) = target {
                if let Some(program) = synthesizer.find_best_program(&rep, target_graph) {
                    hypothesis.program = Some(program.clone());
                }
            }

            if let Some(&(successes, attempts)) = self.representation_stats.get(&name) {
                if attempts > 0 {
                    hypothesis.confidence = successes as f32 / attempts as f32;
                    hypothesis.success_count = successes;
                    hypothesis.total_attempts = attempts;
                }
            }

            self.hypotheses.push(hypothesis);
        }

        self.hypotheses.sort_by(|a, b| {
            b.score().partial_cmp(&a.score()).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn best_hypothesis(&self) -> Option<&Hypothesis> {
        self.hypotheses.iter().filter(|h| h.program.is_some()).next()
    }

    pub fn top_n(&self, n: usize) -> Vec<&Hypothesis> {
        self.hypotheses.iter().take(n).collect()
    }

    pub fn record_success(&mut self, representation_name: &str) {
        if let Some(h) = self.hypotheses.iter_mut().find(|h| h.representation_name == representation_name) {
            h.record_success();
        }
        let stats = self.representation_stats.entry(representation_name.to_string()).or_insert((0, 0));
        stats.0 += 1;
        stats.1 += 1;
    }

    pub fn record_failure(&mut self, representation_name: &str) {
        if let Some(h) = self.hypotheses.iter_mut().find(|h| h.representation_name == representation_name) {
            h.record_failure();
        }
        let stats = self.representation_stats.entry(representation_name.to_string()).or_insert((0, 0));
        stats.1 += 1;
    }

    pub fn best_representation_name(&self) -> Option<&str> {
        self.best_hypothesis().map(|h| h.representation_name.as_str())
    }

    pub fn best_representation(&self) -> Option<&KernelStructureGraph> {
        self.best_hypothesis().map(|h| &h.representation)
    }

    pub fn best_program(&self) -> Option<&Program> {
        self.best_hypothesis().and_then(|h| h.program.as_ref())
    }

    pub fn program_cost(&self, program: &Program) -> f64 {
        program.steps.iter().map(|step| {
            let op_name = match step {
                crate::sandbox::operators::Transformation::Translate { .. } => "Translate",
                crate::sandbox::operators::Transformation::Recolor { .. } => "Recolor",
                crate::sandbox::operators::Transformation::Delete { .. } => "Delete",
                crate::sandbox::operators::Transformation::Create { .. } => "Create",
                crate::sandbox::operators::Transformation::Merge { .. } => "Merge",
                crate::sandbox::operators::Transformation::Split { .. } => "Split",
                crate::sandbox::operators::Transformation::NoOp => "NoOp",
                crate::sandbox::operators::Transformation::RecolorToTarget { .. } => "RecolorToTarget",
                crate::sandbox::operators::Transformation::TranslateToTarget { .. } => "TranslateToTarget",
                crate::sandbox::operators::Transformation::Rotate { .. } => "Rotate",
            };
            self.cost_weights.get(op_name).copied().unwrap_or(1.0)
        }).sum()
    }
}
