use super::{Predicate, PredicateResult};
use crate::structure::KernelStructureGraph;

/// Kiértékel egy predikátumot a megadott gráfon.
pub fn evaluate(predicate: &dyn Predicate, graph: &KernelStructureGraph) -> PredicateResult {
    predicate.evaluate(graph)
}
