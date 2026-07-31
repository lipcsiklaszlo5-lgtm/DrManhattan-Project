use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicEntry {
    pub id: uuid::Uuid,
    pub task_intent: String,
    pub success: bool,
    pub timestamp: u64,
    pub notes: String,
}
