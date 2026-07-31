use crate::sandbox::operators::Transformation;

#[derive(Debug, Clone)]
pub struct TransformRule {
    pub conditions: Vec<Condition>,
    pub action: Transformation,
    pub confidence: f32,
    pub success_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    ColorEquals(String),
    PositionAbove(String),
    PositionLeftOf(String),
    AlwaysTrue,
}

impl TransformRule {
    pub fn new(action: Transformation) -> Self {
        Self { conditions: vec![Condition::AlwaysTrue], action, confidence: 0.5, success_count: 1 }
    }

    pub fn compose(first: &TransformRule, second: &TransformRule) -> Option<TransformRule> {
        Some(TransformRule {
            conditions: first.conditions.clone(),
            action: second.action.clone(),
            confidence: (first.confidence + second.confidence) / 2.0,
            success_count: 0,
        })
    }

    pub fn record_success(&mut self) {
        self.success_count += 1;
        self.confidence = (self.confidence + 0.1).min(1.0);
    }

    pub fn record_failure(&mut self) {
        self.confidence = (self.confidence - 0.1).max(0.0);
    }
}

pub struct TransformationAlgebra {
    pub rules: Vec<TransformRule>,
}

impl TransformationAlgebra {
    pub fn new() -> Self { Self { rules: Vec::new() } }

    pub fn add_rule(&mut self, rule: TransformRule) {
        if let Some(existing) = self.rules.iter_mut().find(|r| r.action == rule.action) {
            existing.record_success();
        } else {
            self.rules.push(rule);
        }
    }

    pub fn find_best(&self, conditions: &[Condition]) -> Option<&TransformRule> {
        self.rules.iter()
            .filter(|r| conditions.iter().all(|c| r.conditions.contains(c)))
            .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
    }
}
