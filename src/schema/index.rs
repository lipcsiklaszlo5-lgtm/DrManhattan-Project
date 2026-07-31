use std::collections::HashMap;
use crate::memory::semantic::{SemanticSchema, Predicate};

/// Fast lookup of schemas by their required/desired predicates.
/// This avoids iterating over all schemas during search.
pub struct SchemaIndex {
    /// Maps a predicate -> list of schema IDs that can handle it
    by_requires: HashMap<Predicate, Vec<uuid::Uuid>>,
    /// Maps a predicate -> list of schema IDs that provide it
    by_provides: HashMap<Predicate, Vec<uuid::Uuid>>,
}

impl SchemaIndex {
    pub fn new() -> Self {
        Self {
            by_requires: HashMap::new(),
            by_provides: HashMap::new(),
        }
    }

    /// Insert a schema into the index.
    pub fn insert(&mut self, schema: &SemanticSchema) {
        let id = schema.id;
        for pred in &schema.algebra.requires {
            self.by_requires.entry(pred.clone()).or_default().push(id);
        }
        for pred in &schema.algebra.provides {
            self.by_provides.entry(pred.clone()).or_default().push(id);
        }
    }

    /// Find schemas that can handle any of the given required predicates.
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

    /// Find schemas that provide any of the given predicates.
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
