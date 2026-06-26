use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}
