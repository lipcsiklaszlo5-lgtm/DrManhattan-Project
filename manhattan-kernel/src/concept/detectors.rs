use super::Concept;
use crate::structure::KernelStructureGraph;
use std::collections::{HashSet, HashMap};

// --- Eredeti, javított detektorok ---

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
        // Hole: node tagged as "hole" OR a node that is contained by another and has smaller area
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
        // Javított: valódi szimmetria keresése
        let objects: Vec<_> = graph.nodes.iter()
            .filter(|n| n.node_type == "arc_object")
            .collect();
        for i in 0..objects.len() {
            for j in (i+1)..objects.len() {
                let a = &objects[i];
                let b = &objects[j];
                if a.attributes.get("color") == b.attributes.get("color") &&
                   a.attributes.get("area") == b.attributes.get("area") &&
                   a.attributes.get("shape_mask") == b.attributes.get("shape_mask") {
                    // Ha van köztük valamilyen térbeli szimmetria-reláció
                    if let (Some(ax), Some(ay), Some(bx), Some(by)) = (
                        a.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()),
                        a.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()),
                        b.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()),
                        b.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()),
                    ) {
                        // Vízszintes vagy függőleges szimmetria?
                        if (ax + a.attributes.get("bbox_w").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0) == bx) ||
                           (ay + a.attributes.get("bbox_h").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0) == by) {
                            return vec![Concept::Symmetry];
                        }
                    }
                }
            }
        }
        vec![]
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

// --- Új detektorok ---

pub struct MirrorDetector;
impl super::ConceptDetector for MirrorDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        let objects: Vec<_> = graph.nodes.iter().filter(|n| n.node_type == "arc_object").collect();
        for i in 0..objects.len() {
            for j in (i+1)..objects.len() {
                let a = &objects[i];
                let b = &objects[j];
                if a.attributes.get("shape_mask") == b.attributes.get("shape_mask") &&
                   a.attributes.get("color") == b.attributes.get("color") {
                    return vec![Concept::Symmetry]; // Tükrözés → szimmetria fogalom
                }
            }
        }
        vec![]
    }
}

pub struct ContainmentDetector;
impl super::ConceptDetector for ContainmentDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        if graph.edges.iter().any(|e| e.rel_type == "contains") {
            vec![Concept::Hole] // Tartalmazás → lyuk fogalom
        } else { vec![] }
    }
}

pub struct AdjacencyDetector;
impl super::ConceptDetector for AdjacencyDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        if graph.edges.iter().any(|e| e.rel_type == "touches") {
            vec![Concept::Connected]
        } else { vec![] }
    }
}

pub struct PatternDetector;
impl super::ConceptDetector for PatternDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        let mut masks = HashSet::new();
        for node in &graph.nodes {
            if let Some(mask) = node.attributes.get("shape_mask") {
                if !masks.insert(mask.clone()) {
                    return vec![Concept::Connected]; // Ismétlődő alakzat → kapcsolódás
                }
            }
        }
        vec![]
    }
}

pub struct CauseEffectDetector;
impl super::ConceptDetector for CauseEffectDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        // Ok-okozat: ha egy node-nak van "old_color" és "new_color" attribútuma is
        for node in &graph.nodes {
            if node.attributes.contains_key("old_color") && node.attributes.contains_key("new_color") {
                return vec![Concept::Player];
            }
        }
        vec![]
    }
}

pub struct SequenceDetector;
impl super::ConceptDetector for SequenceDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        let mut actions = HashMap::new();
        for node in &graph.nodes {
            if let Some(action) = node.attributes.get("action") {
                *actions.entry(action.clone()).or_insert(0) += 1;
            }
        }
        if actions.values().any(|&c| c >= 3) {
            vec![Concept::Connected]
        } else { vec![] }
    }
}

pub struct ObjectCountDetector;
impl super::ConceptDetector for ObjectCountDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        let count = graph.nodes.iter().filter(|n| n.node_type == "arc_object").count();
        if count > 0 { vec![Concept::Connected] } else { vec![] }
    }
}

pub struct FillDetector;
impl super::ConceptDetector for FillDetector {
    fn detect(&self, graph: &KernelStructureGraph) -> Vec<Concept> {
        for edge in &graph.edges {
            if edge.rel_type == "contains" {
                let inner = graph.nodes.iter().find(|n| n.id == edge.to);
                let outer = graph.nodes.iter().find(|n| n.id == edge.from);
                if let (Some(inner), Some(outer)) = (inner, outer) {
                    let inner_area: usize = inner.attributes.get("area").and_then(|v| v.parse().ok()).unwrap_or(0);
                    let outer_area: usize = outer.attributes.get("area").and_then(|v| v.parse().ok()).unwrap_or(0);
                    if inner_area > 0 && inner_area < outer_area {
                        return vec![Concept::Hole, Concept::Border];
                    }
                }
            }
        }
        vec![]
    }
}
