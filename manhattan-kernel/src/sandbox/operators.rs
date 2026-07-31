use crate::structure::KernelStructureGraph;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Transformation {
    RecolorToTarget { node_id: String },
    TranslateToTarget { node_id: String },
    Rotate { node_id: String, angle: u16 },
    Translate { node_id: String, dx: i64, dy: i64 },
    Recolor { node_id: String, new_color: String },
    Delete { node_id: String },
    Create { color: String, bbox_x: u64, bbox_y: u64, bbox_w: u8, bbox_h: u8 },
    Merge { node_a: String, node_b: String },
    Split { node_id: String },
    NoOp,
}

pub fn apply_transformation(graph: &KernelStructureGraph, transform: &Transformation) -> KernelStructureGraph {
    let mut new_graph = graph.clone();
    
    match transform {
        Transformation::Translate { node_id, dx, dy } => {
            if let Some(node) = new_graph.nodes.iter_mut().find(|n| &n.id == node_id) {
                if let (Some(x_str), Some(y_str)) = (
                    node.attributes.get("bbox_x").cloned(),
                    node.attributes.get("bbox_y").cloned(),
                ) {
                    if let (Ok(x), Ok(y)) = (x_str.parse::<i64>(), y_str.parse::<i64>()) {
                        let new_x = (x + dx).max(0);
                        let new_y = (y + dy).max(0);
                        node.attributes.insert("bbox_x".to_string(), new_x.to_string());
                        node.attributes.insert("bbox_y".to_string(), new_y.to_string());
                    }
                }
            }
        }
        Transformation::Recolor { node_id, new_color } => {
            if let Some(node) = new_graph.nodes.iter_mut().find(|n| &n.id == node_id) {
                node.attributes.insert("color".to_string(), new_color.clone());
            }
        }
        Transformation::Delete { node_id } => {
            new_graph.nodes.retain(|n| &n.id != node_id);
            new_graph.edges.retain(|e| &e.from != node_id && &e.to != node_id);
        }
        Transformation::Create { color, bbox_x, bbox_y, bbox_w, bbox_h } => {
            let node_id = format!("obj_{}", new_graph.nodes.len());
            let mut attrs = HashMap::new();
            attrs.insert("color".to_string(), color.clone());
            attrs.insert("bbox_x".to_string(), bbox_x.to_string());
            attrs.insert("bbox_y".to_string(), bbox_y.to_string());
            attrs.insert("bbox_w".to_string(), bbox_w.to_string());
            attrs.insert("bbox_h".to_string(), bbox_h.to_string());
            attrs.insert("area".to_string(), ((*bbox_w as u64) * (*bbox_h as u64)).to_string());
            
            // shape_mask generálása: a bounding box minden pixelét kitöltjük
            let mut shape_mask = Vec::new();
            for dx in 0..*bbox_w {
                for dy in 0..*bbox_h {
                    shape_mask.push(format!("{},{}", dx, dy));
                }
            }
            attrs.insert("shape_mask".to_string(), shape_mask.join(";"));
            
            new_graph.add_node(&node_id, "arc_object");
            if let Some(node) = new_graph.nodes.last_mut() {
                node.attributes = attrs;
            }
        }
        Transformation::Merge { node_a, node_b } => {
            let b_attrs = new_graph.nodes.iter()
                .find(|n| &n.id == node_b)
                .map(|n| n.attributes.clone());
            
            if let Some(b_attr) = b_attrs {
                if let Some(node_a) = new_graph.nodes.iter_mut().find(|n| &n.id == node_a) {
                    if let (Some(ax), Some(ay), Some(aw), Some(ah), Some(bx), Some(by), Some(bw), Some(bh)) = (
                        node_a.attributes.get("bbox_x").and_then(|v| v.parse::<u64>().ok()),
                        node_a.attributes.get("bbox_y").and_then(|v| v.parse::<u64>().ok()),
                        node_a.attributes.get("bbox_w").and_then(|v| v.parse::<u64>().ok()),
                        node_a.attributes.get("bbox_h").and_then(|v| v.parse::<u64>().ok()),
                        b_attr.get("bbox_x").and_then(|v| v.parse::<u64>().ok()),
                        b_attr.get("bbox_y").and_then(|v| v.parse::<u64>().ok()),
                        b_attr.get("bbox_w").and_then(|v| v.parse::<u64>().ok()),
                        b_attr.get("bbox_h").and_then(|v| v.parse::<u64>().ok()),
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
                new_graph.nodes.retain(|n| &n.id != node_b);
                new_graph.edges.retain(|e| &e.from != node_b && &e.to != node_b);
            }
        }
        Transformation::Split { node_id } => {
            if let Some(node) = new_graph.nodes.iter().find(|n| &n.id == node_id).cloned() {
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
                    let id1 = format!("obj_{}", new_graph.nodes.len());
                    new_graph.add_node(&id1, "arc_object");
                    if let Some(new_node) = new_graph.nodes.last_mut() { new_node.attributes = attrs1; }
                    
                    let mut attrs2 = HashMap::new();
                    attrs2.insert("color".to_string(), color);
                    attrs2.insert("bbox_x".to_string(), (bx + half_w as u64).to_string());
                    attrs2.insert("bbox_y".to_string(), by.to_string());
                    attrs2.insert("bbox_w".to_string(), (bw - half_w).to_string());
                    attrs2.insert("bbox_h".to_string(), bh.to_string());
                    let id2 = format!("obj_{}", new_graph.nodes.len());
                    new_graph.add_node(&id2, "arc_object");
                    if let Some(new_node) = new_graph.nodes.last_mut() { new_node.attributes = attrs2; }
                }
                new_graph.nodes.retain(|n| &n.id != node_id);
                new_graph.edges.retain(|e| &e.from != node_id && &e.to != node_id);
            }
        }
        Transformation::NoOp => {}
        Transformation::RecolorToTarget { .. } => { /* absztrakt operátor, a PolicyEngine oldja fel */ }
        Transformation::TranslateToTarget { .. } => { /* absztrakt operátor, a PolicyEngine oldja fel */ }
        Transformation::Rotate { node_id, angle } => {
            if let Some(node) = new_graph.nodes.iter_mut().find(|n| n.id == *node_id) {
                if let (Some(w_str), Some(h_str)) = (node.attributes.get("bbox_w").cloned(), node.attributes.get("bbox_h").cloned()) {
                    if let (Ok(w), Ok(h)) = (w_str.parse::<u8>(), h_str.parse::<u8>()) {
                        if *angle == 90 || *angle == 270 {
                            node.attributes.insert("bbox_w".to_string(), h.to_string());
                            node.attributes.insert("bbox_h".to_string(), w.to_string());
                        }
                    }
                }
            }
        }
    }
    
    new_graph
}

pub fn simulate_plan(graph: &KernelStructureGraph, plan: &[Transformation]) -> Vec<KernelStructureGraph> {
    let mut states = vec![graph.clone()];
    let mut current = graph.clone();
    for transform in plan {
        current = apply_transformation(&current, transform);
        states.push(current.clone());
    }
    states
}

