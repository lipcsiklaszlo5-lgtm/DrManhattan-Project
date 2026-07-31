use crate::structure::KernelStructureGraph;
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ArcOperator {
    Translate { node_id: String, dx: i64, dy: i64 },
    Recolor { node_id: String, new_color: String },
    Delete { node_id: String },
    Create { color: String, bbox_x: u64, bbox_y: u64, bbox_w: u8, bbox_h: u8 },
    Merge { node_a: String, node_b: String },
    Split { node_id: String },
}

pub struct CandidateGenerator {
    pub max_depth: usize,
    pub operator_stats: HashMap<String, (u32, u32)>,
}

impl CandidateGenerator {
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth, operator_stats: HashMap::new() }
    }

    fn success_rate(&self, operator: &str) -> f32 {
        self.operator_stats.get(operator)
            .map(|(s, a)| if *a == 0 { 0.0 } else { *s as f32 / *a as f32 })
            .unwrap_or(0.0)
    }

    /// Régi generate metódus a PolicyEngine számára (kód-specifikus operátorok)
    pub fn generate(&self, base: &KernelStructureGraph, max_candidates: usize) -> Vec<KernelStructureGraph> {
        let mut candidates = Vec::new();
        let mut rng = rand::thread_rng();
        let action = base.nodes.iter()
            .find(|n| n.node_type == "compiler_error")
            .and_then(|n| n.attributes.get("action").cloned())
            .unwrap_or_default();

        match action.as_str() {
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
        candidates
    }

    /// Új generátor ARC operátorokkal
    pub fn generate_arc_candidates(&self, base: &KernelStructureGraph, max_candidates: usize) -> Vec<KernelStructureGraph> {
        let mut candidates = Vec::new();
        let mut rng = rand::thread_rng();
        for _ in 0..max_candidates {
            let mut current = base.clone();
            let depth = rng.gen_range(1..=self.max_depth);
            for _ in 0..depth {
                current = self.apply_arc_operator(&current, &mut rng);
            }
            candidates.push(current);
        }
        candidates
    }

    fn apply_arc_operator(&self, graph: &KernelStructureGraph, rng: &mut impl Rng) -> KernelStructureGraph {
        let mut g = graph.clone();
        if g.nodes.is_empty() { return g; }
        let op_idx = rng.gen_range(0..6);
        match op_idx {
            0 => {
                let idx = rng.gen_range(0..g.nodes.len());
                let node_id = g.nodes[idx].id.clone();
                let dx = rng.gen_range(-3..=3) as i64;
                let dy = rng.gen_range(-3..=3) as i64;
                if let Some(node) = g.nodes.iter_mut().find(|n| n.id == node_id) {
                    if let (Some(x_str), Some(y_str)) = (node.attributes.get("bbox_x").cloned(), node.attributes.get("bbox_y").cloned()) {
                        if let (Ok(x), Ok(y)) = (x_str.parse::<i64>(), y_str.parse::<i64>()) {
                            node.attributes.insert("bbox_x".to_string(), (x + dx).max(0).to_string());
                            node.attributes.insert("bbox_y".to_string(), (y + dy).max(0).to_string());
                        }
                    }
                }
            }
            1 => {
                let idx = rng.gen_range(0..g.nodes.len());
                let node_id = g.nodes[idx].id.clone();
                let colors = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];
                let new_color = colors[rng.gen_range(0..colors.len())].to_string();
                if let Some(node) = g.nodes.iter_mut().find(|n| n.id == node_id) {
                    node.attributes.insert("color".to_string(), new_color);
                }
            }
            2 => {
                let idx = rng.gen_range(0..g.nodes.len());
                let node_id = g.nodes[idx].id.clone();
                g.nodes.retain(|n| n.id != node_id);
                g.edges.retain(|e| e.from != node_id && e.to != node_id);
            }
            3 => {
                let colors = ["1", "2", "3", "4", "5"];
                let color = colors[rng.gen_range(0..colors.len())].to_string();
                let bbox_x = rng.gen_range(0..10) as u64;
                let bbox_y = rng.gen_range(0..10) as u64;
                let bbox_w = rng.gen_range(1..4) as u8;
                let bbox_h = rng.gen_range(1..4) as u8;
                let node_id = format!("obj_{}", g.nodes.len());
                let mut attrs = HashMap::new();
                attrs.insert("color".to_string(), color);
                attrs.insert("bbox_x".to_string(), bbox_x.to_string());
                attrs.insert("bbox_y".to_string(), bbox_y.to_string());
                attrs.insert("bbox_w".to_string(), bbox_w.to_string());
                attrs.insert("bbox_h".to_string(), bbox_h.to_string());
                g.add_node(&node_id, "arc_object");
                if let Some(node) = g.nodes.last_mut() { node.attributes = attrs; }
            }
            4 => {
                if g.nodes.len() >= 2 {
                    let a_idx = rng.gen_range(0..g.nodes.len());
                    let b_idx = loop { let i = rng.gen_range(0..g.nodes.len()); if i != a_idx { break i; } };
                    let node_a_id = g.nodes[a_idx].id.clone();
                    let node_b_id = g.nodes[b_idx].id.clone();
                    let b_attrs = g.nodes[b_idx].attributes.clone();
                    if let Some(node_a) = g.nodes.iter_mut().find(|n| n.id == node_a_id) {
                        if let (Some(ax), Some(ay), Some(aw), Some(ah)) = (
                            node_a.attributes.get("bbox_x").and_then(|v| v.parse::<u64>().ok()),
                            node_a.attributes.get("bbox_y").and_then(|v| v.parse::<u64>().ok()),
                            node_a.attributes.get("bbox_w").and_then(|v| v.parse::<u64>().ok()),
                            node_a.attributes.get("bbox_h").and_then(|v| v.parse::<u64>().ok()),
                        ) {
                            if let (Some(bx), Some(by), Some(bw), Some(bh)) = (
                                b_attrs.get("bbox_x").and_then(|v| v.parse::<u64>().ok()),
                                b_attrs.get("bbox_y").and_then(|v| v.parse::<u64>().ok()),
                                b_attrs.get("bbox_w").and_then(|v| v.parse::<u64>().ok()),
                                b_attrs.get("bbox_h").and_then(|v| v.parse::<u64>().ok()),
                            ) {
                                let new_x = ax.min(bx);
                                let new_y = ay.min(by);
                                let new_w = (ax + aw).max(bx + bw) - new_x;
                                let new_h = (ay + ah).max(by + bh) - new_y;
                                node_a.attributes.insert("bbox_x".to_string(), new_x.to_string());
                                node_a.attributes.insert("bbox_y".to_string(), new_y.to_string());
                                node_a.attributes.insert("bbox_w".to_string(), new_w.to_string());
                                node_a.attributes.insert("bbox_h".to_string(), new_h.to_string());
                            }
                        }
                    }
                    g.nodes.retain(|n| n.id != node_b_id);
                    g.edges.retain(|e| e.from != node_b_id && e.to != node_b_id);
                }
            }
            5 => {
                let idx = rng.gen_range(0..g.nodes.len());
                let node_id = g.nodes[idx].id.clone();
                if let Some(node) = g.nodes.iter().find(|n| n.id == node_id).cloned() {
                    let color = node.attributes.get("color").cloned().unwrap_or("1".to_string());
                    let bx = node.attributes.get("bbox_x").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                    let by = node.attributes.get("bbox_y").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                    let bw = node.attributes.get("bbox_w").and_then(|v| v.parse::<u8>().ok()).unwrap_or(1);
                    let bh = node.attributes.get("bbox_h").and_then(|v| v.parse::<u8>().ok()).unwrap_or(1);
                    let half_w = bw.max(1) / 2;
                    if half_w > 0 {
                        let mut attrs1 = HashMap::new();
                        attrs1.insert("color".to_string(), color.clone());
                        attrs1.insert("bbox_x".to_string(), bx.to_string());
                        attrs1.insert("bbox_y".to_string(), by.to_string());
                        attrs1.insert("bbox_w".to_string(), half_w.to_string());
                        attrs1.insert("bbox_h".to_string(), bh.to_string());
                        let id1 = format!("obj_{}", g.nodes.len());
                        g.add_node(&id1, "arc_object");
                        if let Some(new_node) = g.nodes.last_mut() { new_node.attributes = attrs1; }
                        
                        let mut attrs2 = HashMap::new();
                        attrs2.insert("color".to_string(), color);
                        attrs2.insert("bbox_x".to_string(), (bx + half_w as u64).to_string());
                        attrs2.insert("bbox_y".to_string(), by.to_string());
                        attrs2.insert("bbox_w".to_string(), (bw - half_w).to_string());
                        attrs2.insert("bbox_h".to_string(), bh.to_string());
                        let id2 = format!("obj_{}", g.nodes.len());
                        g.add_node(&id2, "arc_object");
                        if let Some(new_node) = g.nodes.last_mut() { new_node.attributes = attrs2; }
                    }
                    g.nodes.retain(|n| n.id != node_id);
                    g.edges.retain(|e| e.from != node_id && e.to != node_id);
                }
            }
            _ => {}
        }
        g
    }
}
