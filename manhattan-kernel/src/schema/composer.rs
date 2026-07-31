use crate::memory::semantic::{Predicate, SemanticSchema};
use crate::schema::index::SchemaIndex;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct RepairStep {
    pub schema_id: uuid::Uuid,
    pub provides: Vec<Predicate>,
}

pub struct SchemaComposer {
    index: SchemaIndex,
    schemas: HashMap<uuid::Uuid, SemanticSchema>,
}

impl SchemaComposer {
    pub fn new(index: SchemaIndex, schemas: HashMap<uuid::Uuid, SemanticSchema>) -> Self {
        Self { index, schemas }
    }

    pub fn compose(&self, required: &[Predicate]) -> Vec<RepairStep> {
        let mut plan = Vec::new();
        let mut needed: HashSet<Predicate> = required.iter().cloned().collect();

        for _ in 0..10 {
            if needed.is_empty() {
                break;
            }
            let need_vec: Vec<Predicate> = needed.iter().cloned().collect();
            let candidates = self.index.find_by_requires(&need_vec);

            let mut best_step: Option<RepairStep> = None;
            let mut best_score = 0.0;

            for schema_id in &candidates {
                if plan.iter().any(|s: &RepairStep| s.schema_id == *schema_id) {
                    continue;
                }
                if let Some(schema) = self.schemas.get(schema_id) {
                    let overlap = schema.algebra.requires.iter()
                        .filter(|p| needed.contains(p))
                        .count();
                    let confidence = schema.metadata.successes as f32 /
                        (schema.metadata.successes + schema.metadata.failures + 1) as f32;
                    let score = overlap as f32 * confidence;
                    if score > best_score {
                        best_score = score;
                        let provides = schema.algebra.provides.iter().cloned().collect();
                        best_step = Some(RepairStep {
                            schema_id: *schema_id,
                            provides,
                        });
                    }
                }
            }

            if let Some(step) = best_step {
                for p in &step.provides {
                    needed.remove(p);
                }
                if let Some(schema) = self.schemas.get(&step.schema_id) {
                    for p in &schema.algebra.requires {
                        needed.remove(p);
                    }
                }
                plan.push(step);
            } else {
                break;
            }
        }
        plan
    }
}
