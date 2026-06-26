use serde::{Deserialize, Serialize};
use crate::structure::KernelStructureGraph;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSchema {
    pub id: uuid::Uuid,
    pub structure_snapshot: KernelStructureGraph,
    pub confidence: f32,
    pub domain_tags: Vec<String>,
}
