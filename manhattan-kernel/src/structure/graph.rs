use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KernelStructureGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    pub id: String,
    pub node_type: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub rel_type: String,
    pub attributes: HashMap<String, String>,
}

impl KernelStructureGraph {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), edges: Vec::new() }
    }

    pub fn add_node(&mut self, id: &str, node_type: &str) -> &mut Node {
        let node = Node {
            id: id.to_string(),
            node_type: node_type.to_string(),
            attributes: HashMap::new(),
        };
        self.nodes.push(node);
        self.nodes.last_mut().unwrap()
    }

    pub fn add_edge(&mut self, from: &str, to: &str, rel_type: &str) {
        self.edges.push(Edge {
            from: from.to_string(),
            to: to.to_string(),
            rel_type: rel_type.to_string(),
            attributes: HashMap::new(),
        });
    }

    pub fn fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        let mut node_ids: Vec<&String> = self.nodes.iter().map(|n| &n.id).collect();
        node_ids.sort();
        for id in node_ids {
            id.hash(&mut hasher);
        }
        let mut edge_keys: Vec<(&String, &String, &String)> = self.edges.iter()
            .map(|e| (&e.from, &e.to, &e.rel_type)).collect();
        edge_keys.sort();
        for (f, t, r) in edge_keys {
            f.hash(&mut hasher);
            t.hash(&mut hasher);
            r.hash(&mut hasher);
        }
        hasher.finish()
    }
}

pub struct SplitMix64(pub u64);

impl SplitMix64 {
    pub fn new(seed: u64) -> Self { SplitMix64(seed) }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}
