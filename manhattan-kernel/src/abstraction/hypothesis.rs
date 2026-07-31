use crate::structure::KernelStructureGraph;
use crate::adapter::arc::adapter::ArcGrid;
use crate::abstraction::program::{Program, ProgramSynthesizer};
use crate::abstraction::representation::{RepresentationFactory};

#[derive(Debug, Clone)]
pub struct Hypothesis {
    pub representation_name: String,
    pub program: Option<Program>,
    pub cost: f64,
    pub success_count: u32,
    pub total_attempts: u32,
}

impl Hypothesis {
    pub fn new(representation_name: String, program: Option<Program>, cost: f64) -> Self {
        Self { representation_name, program, cost, success_count: 0, total_attempts: 0 }
    }
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 { 0.0 } else { self.success_count as f64 / self.total_attempts as f64 }
    }
    pub fn score(&self) -> f64 {
        let sr = self.success_rate();
        if sr > 0.0 { sr / self.cost.max(0.001) } else { 1.0 / self.cost.max(0.001) }
    }
}

pub struct HypothesisManager {
    pub hypotheses: Vec<Hypothesis>,
    factory: RepresentationFactory,
}

impl HypothesisManager {
    pub fn new() -> Self {
        Self { hypotheses: Vec::new(), factory: RepresentationFactory::new() }
    }
    pub fn process_grid(
        &mut self,
        grid: &ArcGrid,
        synthesizer: &mut ProgramSynthesizer,
        target_ksg: Option<&KernelStructureGraph>,
    ) {
        let reps = self.factory.build_all(grid);
        self.hypotheses.clear();
        for rep in reps {
            let mut program = None;
            let mut cost = 1.0_f64;
            if let Some(target) = target_ksg {
                if let Some(p) = synthesizer.learn_from_example(&rep.graph, target) {
                    cost = p.cost();
                    program = Some(p);
                }
            }
            self.hypotheses.push(Hypothesis::new(rep.name, program, cost));
        }
        self.hypotheses.sort_by(|a, b| a.cost.partial_cmp(&b.cost).unwrap_or(std::cmp::Ordering::Equal));
    }
    pub fn best_hypothesis(&self) -> Option<&Hypothesis> {
        self.hypotheses.first()
    }
    pub fn best_representation_name(&self) -> Option<String> {
        self.best_hypothesis().map(|h| h.representation_name.clone())
    }
    pub fn record_success(&mut self, rep_name: &str) {
        if let Some(h) = self.hypotheses.iter_mut().find(|h| h.representation_name == rep_name) {
            h.success_count += 1; h.total_attempts += 1;
        }
    }
    pub fn record_failure(&mut self, rep_name: &str) {
        if let Some(h) = self.hypotheses.iter_mut().find(|h| h.representation_name == rep_name) {
            h.total_attempts += 1;
        }
    }
    pub fn program_cost(&self, _program: &Program) -> f64 {
        self.hypotheses.first().map(|h| h.cost).unwrap_or(1.0)
    }
}
