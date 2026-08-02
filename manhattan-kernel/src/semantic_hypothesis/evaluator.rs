use crate::structure::KernelStructureGraph;
use super::hypothesis::SemanticHypothesis;

/// Check if a semantic step is consistent with a given input-output pair.
/// Returns true if applying the step to the input produces the expected output (or part of it).
/// For simplicity, we only verify that the selected object matches the expected node id? Not yet.
/// This is a placeholder. Full implementation would simulate the step and compare.
pub fn step_consistent(
    step: &super::hypothesis::SemanticStep,
    input: &KernelStructureGraph,
    _output: &KernelStructureGraph,
    _grid_width: u8,
    _grid_height: u8,
) -> bool {
    // For now, just check if the condition (predicate) matches exactly one node.
    if let Some(conds) = &step.condition {
        for cond in conds {
            let result = cond.evaluate(input);
            match result {
                crate::predicate::PredicateResult::RankedList(ids) if ids.len() == 1 => return true,
                _ => return false,
            }
        }
    }
    false // condition not provided or fails
}

/// Evaluate a hypothesis against all training pairs.
/// Returns the number of pairs the hypothesis is consistent with.
pub fn evaluate_hypothesis(
    hypothesis: &SemanticHypothesis,
    pairs: &[(KernelStructureGraph, KernelStructureGraph, u8, u8)],
) -> usize {
    let mut consistent = 0;
    for (input, output, gw, gh) in pairs {
        let mut all_steps_ok = true;
        for step in &hypothesis.steps {
            if !step_consistent(step, input, output, *gw, *gh) {
                all_steps_ok = false;
                break;
            }
        }
        if all_steps_ok {
            consistent += 1;
        }
    }
    consistent
}
