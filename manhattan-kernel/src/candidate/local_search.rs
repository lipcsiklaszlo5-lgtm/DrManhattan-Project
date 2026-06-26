use crate::structure::KernelStructureGraph;
use rand::Rng;
use std::collections::HashMap;

pub struct CandidateGenerator {
    pub max_depth: usize,
    pub operator_stats: HashMap<String, (u32, u32)>,
}

impl CandidateGenerator {
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            operator_stats: HashMap::new(),
        }
    }

    fn success_rate(&self, operator: &str) -> f32 {
        self.operator_stats.get(operator)
            .map(|(s, a)| if *a == 0 { 0.0 } else { *s as f32 / *a as f32 })
            .unwrap_or(0.0)
    }

    fn sort_by_operator_success(&self, mut candidates: Vec<KernelStructureGraph>) -> Vec<KernelStructureGraph> {
        candidates.sort_by_cached_key(|g| {
            let action = g.nodes.iter()
                .find(|n| n.node_type == "compiler_error")
                .and_then(|n| n.attributes.get("action").cloned())
                .unwrap_or_default();
            -( (self.success_rate(&action) * 1000.0) as i32 )
        });
        candidates
    }

    pub fn generate(&self, base: &KernelStructureGraph, max_candidates: usize) -> Vec<KernelStructureGraph> {
        let mut candidates = Vec::new();
        let mut rng = rand::thread_rng();

        // Az akciót ideiglenes érték nélkül nyerjük ki
        let action = base.nodes.iter()
            .find(|n| n.node_type == "compiler_error")
            .and_then(|n| n.attributes.get("action").cloned())
            .unwrap_or_default();

        match action.as_str() {
            "replace_type" => {
                let type_value_pairs = vec![
                    ("i32", "42"),
                    ("String", "\"hello\""),
                    ("bool", "true"),
                    ("f64", "3.14"),
                    ("char", "'a'"),
                ];
                for i in 0..max_candidates {
                    let mut g = base.clone();
                    let (t, v) = type_value_pairs[rng.gen_range(0..type_value_pairs.len())];
                    let new_type = t.to_string();
                    let new_value = v.to_string();
                    for node in g.nodes.iter_mut() {
                        if node.node_type == "compiler_error" {
                            node.attributes.insert("new_type".into(), new_type.clone());
                            node.attributes.insert("new_value".into(), new_value.clone());
                            node.attributes.insert("variant".into(), format!("v{}", i));
                        }
                    }
                    candidates.push(g);
                }
            }
            "add_import" => {
                let imports = vec![
                    "use std::fmt::Debug;",
                    "use std::io;",
                    "use std::path::Path;",
                    "use std::collections::HashMap;",
                ];
                for _i in 0..max_candidates {
                    let mut g = base.clone();
                    let chosen = imports[rng.gen_range(0..imports.len())].to_string();
                    for node in g.nodes.iter_mut() {
                        if node.node_type == "compiler_error" {
                            node.attributes.insert("annotation".into(), chosen.clone());
                        }
                    }
                    candidates.push(g);
                }
            }
            "rename" => {
                let names = vec!["x", "y", "z", "val", "data"];
                for _i in 0..max_candidates {
                    let mut g = base.clone();
                    let new_name = names[rng.gen_range(0..names.len())].to_string();
                    for node in g.nodes.iter_mut() {
                        if node.node_type == "compiler_error" {
                            node.attributes.insert("new_name".into(), new_name.clone());
                        }
                    }
                    candidates.push(g);
                }
            }
            "delete_line" => {
                for i in 0..max_candidates {
                    let mut g = base.clone();
                    let line = (i % 5 + 1).to_string();
                    for node in g.nodes.iter_mut() {
                        if node.node_type == "compiler_error" {
                            node.attributes.insert("line".into(), line.clone());
                        }
                    }
                    candidates.push(g);
                }
            }
            "fix_main" => {
                for i in 0..max_candidates {
                    let mut g = base.clone();
                    let marker = format!("variant_{}", i);
                    for node in g.nodes.iter_mut() {
                        if node.node_type == "compiler_error" {
                            node.attributes.insert("variant".into(), marker.clone());
                        }
                    }
                    candidates.push(g);
                }
            }
            _ => {
                for edge in &base.edges {
                    if candidates.len() >= max_candidates { break; }
                    let mut g = base.clone();
                    g.edges.retain(|e| !(e.from == edge.from && e.to == edge.to && e.rel_type == edge.rel_type));
                    candidates.push(g);
                }
                while candidates.len() < max_candidates {
                    candidates.push(base.clone());
                }
            }
        }
        candidates.truncate(max_candidates);
        self.sort_by_operator_success(candidates)
    }
}
