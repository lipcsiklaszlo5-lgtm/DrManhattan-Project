use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralRule {
    pub id: uuid::Uuid,
    pub pattern: String,
    pub confidence: f32,
    pub success_count: u32,
    pub domain_tags: Vec<String>,
}

impl ProceduralRule {
    pub fn record_success(&mut self) {
        self.success_count += 1;
        self.confidence = (self.confidence + 0.05).min(1.0);
    }
    pub fn record_failure(&mut self) {
        self.confidence = (self.confidence - 0.1).max(0.0);
    }
}
