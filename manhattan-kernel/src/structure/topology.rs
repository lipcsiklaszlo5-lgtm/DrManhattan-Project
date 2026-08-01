use crate::structure::{KernelStructureGraph, Node};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeTransformation {
    Translate { node_id: String, dx: i64, dy: i64 },
    Recolor { node_id: String, new_color: String },
    Delete { node_id: String },
    Create { node_id: String, color: String, bbox_x: u64, bbox_y: u64 },
    Rotate { node_id: String, angle: u16 },
    Unchanged { node_id: String },
}

fn wl_iteration(graph: &KernelStructureGraph, colors: &mut HashMap<String, u64>) {
    let mut new_colors: HashMap<String, Vec<u64>> = HashMap::new();
    for node in &graph.nodes {
        let mut neighbor_colors: Vec<u64> = graph.edges.iter()
            .filter(|e| e.from == node.id || e.to == node.id)
            .map(|e| {
                let neighbor_id = if e.from == node.id { &e.to } else { &e.from };
                *colors.get(neighbor_id).unwrap_or(&0)
            })
            .collect();
        neighbor_colors.sort();
        new_colors.insert(node.id.clone(), neighbor_colors);
    }

    let mut color_map: HashMap<Vec<u64>, u64> = HashMap::new();
    let mut next_color = 1u64;
    for node in &graph.nodes {
        let neighbors = new_colors.get(&node.id).cloned().unwrap_or_default();
        let new_color = *color_map.entry(neighbors).or_insert_with(|| {
            let c = next_color;
            next_color += 1;
            c
        });
        colors.insert(node.id.clone(), new_color);
    }
}

pub fn wl_hash(graph: &KernelStructureGraph, iterations: usize) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut colors: HashMap<String, u64> = HashMap::new();
    for node in &graph.nodes {
        let mut hasher = DefaultHasher::new();
        node.node_type.hash(&mut hasher);
        for (k, v) in &node.attributes {
            k.hash(&mut hasher);
            v.hash(&mut hasher);
        }
        colors.insert(node.id.clone(), hasher.finish());
    }

    for _ in 0..iterations {
        wl_iteration(graph, &mut colors);
    }

    let mut sorted_colors: Vec<u64> = colors.values().cloned().collect();
    sorted_colors.sort();
    let mut hasher = DefaultHasher::new();
    for c in &sorted_colors {
        c.hash(&mut hasher);
    }
    let mut node_ids: Vec<&String> = graph.nodes.iter().map(|n| &n.id).collect();
    node_ids.sort();
    for id in node_ids {
        if let Some(node) = graph.nodes.iter().find(|n| &n.id == id) {
            let mut attr_keys: Vec<&String> = node.attributes.keys().collect();
            attr_keys.sort();
            for k in attr_keys {
                if let Some(v) = node.attributes.get(k) {
                    k.hash(&mut hasher);
                    v.hash(&mut hasher);
                }
            }
        }
    }
    hasher.finish()
}

pub fn graph_diff(
    before: &KernelStructureGraph,
    after: &KernelStructureGraph,
) -> Vec<NodeTransformation> {
    let mut transformations = Vec::new();
    let before_nodes: HashMap<String, &Node> = before.nodes.iter().map(|n| (n.id.clone(), n)).collect();
    let after_nodes: HashMap<String, &Node> = after.nodes.iter().map(|n| (n.id.clone(), n)).collect();

    let before_ids: HashSet<&String> = before_nodes.keys().collect();
    let after_ids: HashSet<&String> = after_nodes.keys().collect();

    for id in after_ids.difference(&before_ids) {
        if let Some(node) = after_nodes.get(*id) {
            let color = node.attributes.get("color").cloned().unwrap_or_default();
            let bbox_x = node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
            let bbox_y = node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
            transformations.push(NodeTransformation::Create {
                node_id: id.to_string(),
                color,
                bbox_x,
                bbox_y,
            });
        }
    }

    for id in before_ids.difference(&after_ids) {
        transformations.push(NodeTransformation::Delete {
            node_id: id.to_string(),
        });
    }

    for id in before_ids.intersection(&after_ids) {
        let before_node = before_nodes[*id];
        let after_node = after_nodes[*id];

        let before_color = before_node.attributes.get("color").cloned().unwrap_or_default();
        let after_color = after_node.attributes.get("color").cloned().unwrap_or_default();
        if before_color != after_color {
            transformations.push(NodeTransformation::Recolor {
                node_id: id.to_string(),
                new_color: after_color.clone(),
            });
        }

        let bx: i64 = before_node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
        let by: i64 = before_node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
        let ax: i64 = after_node.attributes.get("bbox_x").and_then(|v| v.parse().ok()).unwrap_or(0);
        let ay: i64 = after_node.attributes.get("bbox_y").and_then(|v| v.parse().ok()).unwrap_or(0);
        if bx != ax || by != ay {
            transformations.push(NodeTransformation::Translate {
                node_id: id.to_string(),
                dx: ax - bx,
                dy: ay - by,
            });
        }

        // Rotate detektálása: ha a szélesség és magasság felcserélődött
        if let (Some(b_w), Some(b_h), Some(a_w), Some(a_h)) = (
            before_node.attributes.get("bbox_w").and_then(|v| v.parse::<u8>().ok()),
            before_node.attributes.get("bbox_h").and_then(|v| v.parse::<u8>().ok()),
            after_node.attributes.get("bbox_w").and_then(|v| v.parse::<u8>().ok()),
            after_node.attributes.get("bbox_h").and_then(|v| v.parse::<u8>().ok()),
        ) {
            if b_w == a_h && b_h == a_w && b_w != b_h {
                transformations.push(NodeTransformation::Rotate {
                    node_id: id.to_string(),
                    angle: 90,
                });
            }
        }

        if before_color == after_color && bx == ax && by == ay {
            // Csak akkor Unchanged, ha semmi más nem változott
            let w_changed = before_node.attributes.get("bbox_w") != after_node.attributes.get("bbox_w");
            let h_changed = before_node.attributes.get("bbox_h") != after_node.attributes.get("bbox_h");
            let rotated = w_changed && h_changed && 
                before_node.attributes.get("bbox_w") == after_node.attributes.get("bbox_h") &&
                before_node.attributes.get("bbox_h") == after_node.attributes.get("bbox_w");
            if !rotated {
                transformations.push(NodeTransformation::Unchanged {
                    node_id: id.to_string(),
                });
            }
        }
    }

    transformations
}

pub fn node_wl_hash(node: &Node, graph: &KernelStructureGraph) -> u64 {
    let mut hash = 0u64;
    for (key, val) in &node.attributes {
        hash ^= (key.len() as u64) << 16 | val.len() as u64;
    }
    hash
}
