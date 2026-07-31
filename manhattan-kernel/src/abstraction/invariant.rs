use crate::structure::KernelStructureGraph;

pub struct InvariantDetector;

impl InvariantDetector {
    pub fn extract_rule(before: &KernelStructureGraph, after: &KernelStructureGraph) -> Option<String> {
        let mut rules = Vec::new();

        for after_node in &after.nodes {
            let before_node = before.nodes.iter().find(|n| n.id == after_node.id);

            match before_node {
                Some(bn) => {
                    let before_color = bn.attributes.get("color").cloned().unwrap_or_default();
                    let after_color = after_node.attributes.get("color").cloned().unwrap_or_default();
                    if before_color != after_color {
                        rules.push(format!("Recolor({}, {})", after_node.id, after_color));
                    }

                    let bx = bn.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                    let by = bn.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                    let ax = after_node.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                    let ay = after_node.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                    if bx != ax || by != ay {
                        rules.push(format!("Translate({}, {}, {})", after_node.id, ax - bx, ay - by));
                    }
                }
                None => {
                    let color = after_node.attributes.get("color").cloned().unwrap_or_default();
                    rules.push(format!("Create({})", color));
                }
            }
        }

        for before_node in &before.nodes {
            if !after.nodes.iter().any(|n| n.id == before_node.id) {
                rules.push(format!("Delete({})", before_node.id));
            }
        }

        if rules.is_empty() { None } else { Some(rules.join(" → ")) }
    }
}
