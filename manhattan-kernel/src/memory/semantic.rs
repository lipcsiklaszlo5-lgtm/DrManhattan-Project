use serde::{Deserialize, Serialize};
use crate::structure::KernelStructureGraph;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Predicate {
    TypeMismatch,
    MissingImport,
    BorrowConflict,
    LifetimeError,
    UnresolvedName,
    TypeResolved,
    ImportResolved,
    BorrowResolved,
    LifetimeResolved,
    NameResolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaAlgebra {
    pub requires: HashSet<Predicate>,
    pub provides: HashSet<Predicate>,
    pub modifies: Vec<String>,
}

impl Default for SchemaAlgebra {
    fn default() -> Self {
        Self {
            requires: HashSet::new(),
            provides: HashSet::new(),
            modifies: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMetadata {
    pub successes: u64,
    pub failures: u64,
    pub avg_cost: f32,
    pub compiler_errors: Vec<String>,
    pub fingerprint: u64,
    pub tags: Vec<String>,
}

impl Default for SchemaMetadata {
    fn default() -> Self {
        Self {
            successes: 0,
            failures: 0,
            avg_cost: 0.0,
            compiler_errors: vec![],
            fingerprint: 0,
            tags: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSchema {
    pub id: uuid::Uuid,
    pub graph: KernelStructureGraph,
    pub metadata: SchemaMetadata,
    pub algebra: SchemaAlgebra,
    pub confidence: f32,
    pub domain_tags: Vec<String>,
}

impl SemanticSchema {
    pub fn new(graph: KernelStructureGraph) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            graph,
            metadata: SchemaMetadata::default(),
            algebra: SchemaAlgebra::default(),
            confidence: 0.5,
            domain_tags: vec![],
        }
    }
}
