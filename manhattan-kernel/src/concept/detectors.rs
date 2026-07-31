use super::Concept;
use crate::structure::KernelStructureGraph;
use std::collections::HashSet;

pub struct BorderDetector;
impl super::ConceptDetector for BorderDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        if graph.nodes.iter().any(|n| n.attributes.get("role").map_or(false, |v| v == "border")) {
            vec![Concept::Border]
        } else { vec![] }
    }
}

pub struct HoleDetector;
impl super::ConceptDetector for HoleDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        if graph.nodes.iter().any(|n| n.attributes.get("shape").map_or(false, |v| v == "hole")) {
            return vec![Concept::Hole];
        }
        for edge in &graph.edges {
            if edge.rel_type == "contains" {
                let inner = graph.nodes.iter().find(|n| n.id == edge.to);
                let outer = graph.nodes.iter().find(|n| n.id == edge.from);
                if let (Some(inner), Some(outer)) = (inner, outer) {
                    let inner_area: usize = inner.attributes.get("area").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let outer_area: usize = outer.attributes.get("area").and_then(|v| v.parse().ok()).unwrap_or(0);
                    if inner_area > 0 && inner_area < outer_area {
                        return vec![Concept::Hole];
                    }
                }
            }
        }
        vec![]
    }
}

pub struct SymmetryDetector;
impl super::ConceptDetector for SymmetryDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        if graph.nodes.iter().any(|n| n.attributes.contains_key("symmetry")) {
            vec![Concept::Symmetry]
        } else { vec![] }
    }
}

pub struct LargestObjectDetector;
impl super::ConceptDetector for LargestObjectDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        let largest = graph.nodes.iter()
            .filter(|n| n.node_type == "arc_object")
            .max_by_key(|n| n.attributes.get("area").and_then(|v| v.parse::<usize>().ok()).unwrap_or(0));
        if largest.is_some() { vec![Concept::Largest] } else { vec![] }
    }
}

pub struct CrossDetector;
impl super::ConceptDetector for CrossDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        for node in &graph.nodes {
            if let Some(mask_str) = node.attributes.get("shape_mask") {
                let pixels: HashSet<(i32,i32)> = mask_str.split(';')
                    .filter_map(|s| {
                        let mut it = s.split(',');
                        let dx: i32 = it.next()?.parse().ok()?;
                        let dy: i32 = it.next()?.parse().ok()?;
                        Some((dx, dy))
                    }).collect();
                let has_cross = (0..5).all(|i| pixels.contains(&(i,2)))
                    && (0..5).all(|i| pixels.contains(&(2,i)));
                if has_cross && pixels.len() >= 5 {
                    return vec![Concept::Cross];
                }
            }
        }
        vec![]
    }
}

pub struct RoleDetector;
impl super::ConceptDetector for RoleDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        let mut res = Vec::new();
        for n in &graph.nodes {
            if let Some(role) = n.attributes.get("role") {
                match role.as_str() {
                    "player" => res.push(Concept::Player),
                    "exit" => res.push(Concept::Exit),
                    "key" => res.push(Concept::Key),
                    "door" => res.push(Concept::Door),
                    "button" => res.push(Concept::Button),
                    "obstacle" => res.push(Concept::Obstacle),
                    _ => {}
                }
            }
        }
        res
    }
}
