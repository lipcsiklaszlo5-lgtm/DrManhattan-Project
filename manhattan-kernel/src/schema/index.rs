use std::collections::HashMap;
use crate::memory::semantic::{SemanticSchema, Predicate};

#[derive(Clone)]
pub struct SchemaIndex {
    by_requires: HashMap<Predicate, Vec<uuid::Uuid>>,
    by_provides: HashMap<Predicate, Vec<uuid::Uuid>>,
}

impl SchemaIndex {
    pub fn new() -> Self {
        Self {
            by_requires: HashMap::new(),
            by_provides: HashMap::new(),
        }
    }

    pub fn insert(&mut self, schema: &SemanticSchema) {
        let id = schema.id;
        for pred in &schema.algebra.requires {
            self.by_requires.entry(pred.clone()).or_default().push(id);
        }
        for pred in &schema.algebra.provides {
            self.by_provides.entry(pred.clone()).or_default().push(id);
        }
    }

    pub fn find_by_requires(&self, predicates: &[Predicate]) -> Vec<uuid::Uuid> {
        let mut ids = Vec::new();
        for pred in predicates {
            if let Some(list) = self.by_requires.get(pred) {
                ids.extend_from_slice(list);
            }
        }
        ids.sort();
        ids.dedup();
        ids
    }

    pub fn find_by_provides(&self, predicates: &[Predicate]) -> Vec<uuid::Uuid> {
        let mut ids = Vec::new();
        for pred in predicates {
            if let Some(list) = self.by_provides.get(pred) {
                ids.extend_from_slice(list);
            }
        }
        ids.sort();
        ids.dedup();
        ids
    }
}
