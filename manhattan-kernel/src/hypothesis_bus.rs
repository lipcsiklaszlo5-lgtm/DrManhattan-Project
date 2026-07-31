//! Hypothesis bus – Bonsai (LLM) concept proposal channel
//! The kernel can query concepts and reject/accept them.

use std::collections::VecDeque;

/// A single concept hypothesis from Bonsai.
#[derive(Debug, Clone, PartialEq)]
pub struct BonsaiHypothesis {
    pub concept: String,
    pub confidence: f64,
    pub evidence: String,
}

/// The bus: a queue of hypotheses submitted by Bonsai.
/// The kernel pulls them via get_hypotheses() and resets.
#[derive(Debug, Default)]
pub struct HypothesisBus {
    hypotheses: VecDeque<BonsaiHypothesis>,
}

impl HypothesisBus {
    pub fn new() -> Self {
        Self {
            hypotheses: VecDeque::new(),
        }
    }

    /// Submit a hypothesis (from Bonsai adapter).
    pub fn submit(&mut self, hypothesis: BonsaiHypothesis) {
        self.hypotheses.push_back(hypothesis);
    }

    /// Get all pending hypotheses and clear the bus.
    pub fn get_hypotheses(&mut self) -> Vec<BonsaiHypothesis> {
        self.hypotheses.drain(..).collect()
    }

    /// Peek without consuming.
    pub fn peek(&self) -> &VecDeque<BonsaiHypothesis> {
        &self.hypotheses
    }

    /// True if empty.
    pub fn is_empty(&self) -> bool {
        self.hypotheses.is_empty()
    }
}
