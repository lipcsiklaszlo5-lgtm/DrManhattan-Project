pub mod builtin;
pub mod evaluator;

use crate::structure::KernelStructureGraph;

/// A predikátum kiértékelésének eredménye.
#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub enum PredicateResult {
    Bool(bool),
    RankedList(Vec<(String, f32)>),
}

impl PredicateResult {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PredicateResult::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_ranked_list(&self) -> Vec<(String, f32)> {
        match self {
            PredicateResult::RankedList(list) => list.clone(),
            _ => Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            PredicateResult::RankedList(list) => list.len(),
            PredicateResult::Bool(true) => 1,
            PredicateResult::Bool(false) => 0,
        }
    }

    pub fn is_true(&self) -> bool {
        match self {
            PredicateResult::Bool(b) => *b,
            PredicateResult::RankedList(list) => !list.is_empty(),
        }
    }
}

/// Közös interfész minden predikátumhoz.
pub trait Predicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult;
    fn name(&self) -> &str;
    fn required_attributes(&self) -> Vec<String> { Vec::new() }
    /// Klónoz egy dobozolt predikátumot.
    fn clone_box(&self) -> Box<dyn Predicate>;
}
#[cfg(test)] pub mod tests;
