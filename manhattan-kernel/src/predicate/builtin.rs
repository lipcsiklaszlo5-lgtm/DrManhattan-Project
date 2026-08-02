use super::{Predicate, PredicateResult};
use crate::structure::KernelStructureGraph;
use std::collections::HashSet;

// ---------------------------------------------------------------------
// Attribútum predikátumok
// ---------------------------------------------------------------------
pub struct ColorPredicate { pub color: String }

impl Predicate for ColorPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| n.attributes.get("color") == Some(&self.color))
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "ColorPredicate" }
    fn required_attributes(&self) -> Vec<String> { vec!["color".to_string()] }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(ColorPredicate { color: self.color.clone() }) }
}

pub struct AreaPredicate { pub min: Option<u64>, pub max: Option<u64> }

impl Predicate for AreaPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter_map(|n| {
                let area = n.attributes.get("area").and_then(|v| v.parse::<u64>().ok())?;
                let ok = self.min.map_or(true, |m| area >= m) && self.max.map_or(true, |m| area <= m);
                if ok { Some((n.id.clone(), 1.0)) } else { None }
            }).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "AreaPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(AreaPredicate { min: self.min, max: self.max }) }
}

pub struct WidthPredicate { pub width: u64 }

impl Predicate for WidthPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| n.attributes.get("bbox_w").and_then(|v| v.parse::<u64>().ok()) == Some(self.width))
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "WidthPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(WidthPredicate { width: self.width }) }
}

pub struct HeightPredicate { pub height: u64 }

impl Predicate for HeightPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| n.attributes.get("bbox_h").and_then(|v| v.parse::<u64>().ok()) == Some(self.height))
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "HeightPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(HeightPredicate { height: self.height }) }
}

pub struct RolePredicate { pub role: String }

impl Predicate for RolePredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| n.attributes.get("role") == Some(&self.role))
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "RolePredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(RolePredicate { role: self.role.clone() }) }
}

pub struct ShapePredicate { pub mask: String }

impl Predicate for ShapePredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| n.attributes.get("shape_mask") == Some(&self.mask))
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "ShapePredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(ShapePredicate { mask: self.mask.clone() }) }
}

pub struct PixelCountPredicate { pub count: usize }

impl Predicate for PixelCountPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter_map(|n| {
                let pixels: Vec<&str> = n.attributes.get("shape_mask")?.split(';').collect();
                if pixels.len() == self.count { Some((n.id.clone(), 1.0)) } else { None }
            }).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "PixelCountPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(PixelCountPredicate { count: self.count }) }
}

pub struct AspectRatioPredicate { pub ratio: f32 }

impl Predicate for AspectRatioPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter_map(|n| {
                let w = n.attributes.get("bbox_w").and_then(|v| v.parse::<f32>().ok())?;
                let h = n.attributes.get("bbox_h").and_then(|v| v.parse::<f32>().ok())?;
                if h > 0.0 && (w / h - self.ratio).abs() < 0.01 { Some((n.id.clone(), 1.0)) } else { None }
            }).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "AspectRatioPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(AspectRatioPredicate { ratio: self.ratio }) }
}

// ---------------------------------------------------------------------
// Globális predikátumok
// ---------------------------------------------------------------------
pub struct LargestPredicate;
impl Predicate for LargestPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let objects: Vec<_> = graph.nodes.iter().filter(|n| n.node_type == "arc_object").collect();
        if objects.is_empty() { return PredicateResult::Bool(false); }
        let max_area = objects.iter().filter_map(|n| n.attributes.get("area").and_then(|v| v.parse::<usize>().ok())).max().unwrap_or(0);
        let largest: Vec<(String, f32)> = objects.into_iter()
            .filter(|n| n.attributes.get("area").and_then(|v| v.parse::<usize>().ok()) == Some(max_area))
            .map(|n| (n.id.clone(), 1.0)).collect();
        PredicateResult::RankedList(largest)
    }
    fn name(&self) -> &str { "LargestPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(LargestPredicate) }
}

pub struct SmallestPredicate;
impl Predicate for SmallestPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let objects: Vec<_> = graph.nodes.iter().filter(|n| n.node_type == "arc_object").collect();
        if objects.is_empty() { return PredicateResult::Bool(false); }
        let min_area = objects.iter().filter_map(|n| n.attributes.get("area").and_then(|v| v.parse::<usize>().ok())).min().unwrap_or(0);
        let smallest: Vec<(String, f32)> = objects.into_iter()
            .filter(|n| n.attributes.get("area").and_then(|v| v.parse::<usize>().ok()) == Some(min_area))
            .map(|n| (n.id.clone(), 1.0)).collect();
        PredicateResult::RankedList(smallest)
    }
    fn name(&self) -> &str { "SmallestPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(SmallestPredicate) }
}

pub struct LeftmostPredicate;
impl Predicate for LeftmostPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let min_x = graph.nodes.iter().filter_map(|n| n.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok())).min();
        match min_x {
            Some(mx) => {
                let matching: Vec<(String, f32)> = graph.nodes.iter()
                    .filter(|n| n.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()) == Some(mx))
                    .map(|n| (n.id.clone(), 1.0)).collect();
                PredicateResult::RankedList(matching)
            }
            None => PredicateResult::Bool(false)
        }
    }
    fn name(&self) -> &str { "LeftmostPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(LeftmostPredicate) }
}

pub struct RightmostPredicate;
impl Predicate for RightmostPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let max_right = graph.nodes.iter().filter_map(|n| {
            let x = n.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok())?;
            let w = n.attributes.get("bbox_w").and_then(|v| v.parse::<i64>().ok())?;
            Some(x + w)
        }).max();
        match max_right {
            Some(mr) => {
                let matching: Vec<(String, f32)> = graph.nodes.iter()
                    .filter_map(|n| {
                        let x = n.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok())?;
                        let w = n.attributes.get("bbox_w").and_then(|v| v.parse::<i64>().ok())?;
                        if x + w == mr { Some((n.id.clone(), 1.0)) } else { None }
                    }).collect();
                PredicateResult::RankedList(matching)
            }
            None => PredicateResult::Bool(false)
        }
    }
    fn name(&self) -> &str { "RightmostPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(RightmostPredicate) }
}

pub struct TopmostPredicate;
impl Predicate for TopmostPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let min_y = graph.nodes.iter().filter_map(|n| n.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok())).min();
        match min_y {
            Some(my) => {
                let matching: Vec<(String, f32)> = graph.nodes.iter()
                    .filter(|n| n.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()) == Some(my))
                    .map(|n| (n.id.clone(), 1.0)).collect();
                PredicateResult::RankedList(matching)
            }
            None => PredicateResult::Bool(false)
        }
    }
    fn name(&self) -> &str { "TopmostPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(TopmostPredicate) }
}

pub struct BottommostPredicate;
impl Predicate for BottommostPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let max_bottom = graph.nodes.iter().filter_map(|n| {
            let y = n.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok())?;
            let h = n.attributes.get("bbox_h").and_then(|v| v.parse::<i64>().ok())?;
            Some(y + h)
        }).max();
        match max_bottom {
            Some(mb) => {
                let matching: Vec<(String, f32)> = graph.nodes.iter()
                    .filter_map(|n| {
                        let y = n.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok())?;
                        let h = n.attributes.get("bbox_h").and_then(|v| v.parse::<i64>().ok())?;
                        if y + h == mb { Some((n.id.clone(), 1.0)) } else { None }
                    }).collect();
                PredicateResult::RankedList(matching)
            }
            None => PredicateResult::Bool(false)
        }
    }
    fn name(&self) -> &str { "BottommostPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(BottommostPredicate) }
}

pub struct OnlyObjectPredicate;
impl Predicate for OnlyObjectPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let count = graph.nodes.iter().filter(|n| n.node_type == "arc_object").count();
        PredicateResult::Bool(count == 1)
    }
    fn name(&self) -> &str { "OnlyObjectPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(OnlyObjectPredicate) }
}

pub struct UniqueColorPredicate;
impl Predicate for UniqueColorPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let mut color_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for n in &graph.nodes {
            if let Some(c) = n.attributes.get("color") {
                *color_counts.entry(c.as_str()).or_insert(0) += 1;
            }
        }
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| n.attributes.get("color").map_or(false, |c| color_counts.get(c.as_str()) == Some(&1)))
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "UniqueColorPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(UniqueColorPredicate) }
}

pub struct MajorityColorPredicate;
impl Predicate for MajorityColorPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let mut color_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for n in &graph.nodes {
            if let Some(c) = n.attributes.get("color") {
                *color_counts.entry(c.as_str()).or_insert(0) += 1;
            }
        }
        let max_color = color_counts.iter().max_by_key(|(_, &cnt)| cnt).map(|(c, _)| *c);
        match max_color {
            Some(mc) => {
                let matching: Vec<(String, f32)> = graph.nodes.iter()
                    .filter(|n| n.attributes.get("color") == Some(&mc.to_string()))
                    .map(|n| (n.id.clone(), 1.0)).collect();
                PredicateResult::RankedList(matching)
            }
            None => PredicateResult::Bool(false)
        }
    }
    fn name(&self) -> &str { "MajorityColorPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(MajorityColorPredicate) }
}

pub struct MinorityColorPredicate;
impl Predicate for MinorityColorPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let mut color_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for n in &graph.nodes {
            if let Some(c) = n.attributes.get("color") {
                *color_counts.entry(c.as_str()).or_insert(0) += 1;
            }
        }
        let min_color = color_counts.iter().min_by_key(|(_, &cnt)| cnt).map(|(c, _)| *c);
        match min_color {
            Some(mc) => {
                let matching: Vec<(String, f32)> = graph.nodes.iter()
                    .filter(|n| n.attributes.get("color") == Some(&mc.to_string()))
                    .map(|n| (n.id.clone(), 1.0)).collect();
                PredicateResult::RankedList(matching)
            }
            None => PredicateResult::Bool(false)
        }
    }
    fn name(&self) -> &str { "MinorityColorPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(MinorityColorPredicate) }
}

pub struct CenterObjectPredicate;
impl Predicate for CenterObjectPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let nodes: Vec<_> = graph.nodes.iter().filter(|n| n.node_type == "arc_object").collect();
        if nodes.is_empty() { return PredicateResult::Bool(false); }
        // Középpont: a gráf bbox közepének közelítése a node-ok bbox koordinátáiból
        let cx = nodes.iter().filter_map(|n| n.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok())).sum::<i64>() / nodes.len() as i64;
        let cy = nodes.iter().filter_map(|n| n.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok())).sum::<i64>() / nodes.len() as i64;
        let mut scored: Vec<(String, f32)> = nodes.iter()
            .map(|n| {
                let x = n.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                let y = n.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                let dist = ((x - cx).pow(2) + (y - cy).pow(2)) as f32;
                (n.id.clone(), 1.0 / (1.0 + dist.sqrt()))
            }).collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        PredicateResult::RankedList(scored)
    }
    fn name(&self) -> &str { "CenterObjectPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(CenterObjectPredicate) }
}

pub struct CornerObjectPredicate;
impl Predicate for CornerObjectPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| {
                let x = n.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                let y = n.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                x == 0 || y == 0
            }).map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "CornerObjectPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(CornerObjectPredicate) }
}

// BorderObjectPredicate már létezik BorderPredicate néven, átnevezzük.
pub struct BorderObjectPredicate;
impl Predicate for BorderObjectPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| {
                let x = n.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                let y = n.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                x <= 0 || y <= 0
            }).map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "BorderObjectPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(BorderObjectPredicate) }
}

// ---------------------------------------------------------------------
// Alakzat predikátumok
// ---------------------------------------------------------------------
pub struct SymmetryPredicate;
impl Predicate for SymmetryPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let objects: Vec<_> = graph.nodes.iter().filter(|n| n.node_type == "arc_object").collect();
        for i in 0..objects.len() {
            for j in (i+1)..objects.len() {
                let a = &objects[i]; let b = &objects[j];
                if a.attributes.get("color") == b.attributes.get("color") &&
                   a.attributes.get("shape_mask") == b.attributes.get("shape_mask") {
                    return PredicateResult::RankedList(vec![(a.id.clone(), 1.0), (b.id.clone(), 1.0)]);
                }
            }
        }
        PredicateResult::Bool(false)
    }
    fn name(&self) -> &str { "SymmetryPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(SymmetryPredicate) }
}

pub struct RectanglePredicate;
impl Predicate for RectanglePredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| {
                if let Some(mask) = n.attributes.get("shape_mask") {
                    let pixels: Vec<(i32,i32)> = mask.split(';').filter_map(|s| {
                        let mut it = s.split(','); Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?))
                    }).collect();
                    let min_x = pixels.iter().map(|p| p.0).min().unwrap_or(0);
                    let max_x = pixels.iter().map(|p| p.0).max().unwrap_or(0);
                    let min_y = pixels.iter().map(|p| p.1).min().unwrap_or(0);
                    let max_y = pixels.iter().map(|p| p.1).max().unwrap_or(0);
                    let area = ((max_x - min_x + 1) * (max_y - min_y + 1)) as usize;
                    pixels.len() == area
                } else { false }
            }).map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "RectanglePredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(RectanglePredicate) }
}

pub struct LinePredicate;
impl Predicate for LinePredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| {
                if let Some(mask) = n.attributes.get("shape_mask") {
                    let pixels: Vec<(i32,i32)> = mask.split(';').filter_map(|s| {
                        let mut it = s.split(','); Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?))
                    }).collect();
                    let xs: Vec<i32> = pixels.iter().map(|p| p.0).collect();
                    let ys: Vec<i32> = pixels.iter().map(|p| p.1).collect();
                    xs.iter().all(|&x| x == xs[0]) || ys.iter().all(|&y| y == ys[0])
                } else { false }
            }).map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "LinePredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(LinePredicate) }
}

pub struct PointPredicate;
impl Predicate for PointPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| n.attributes.get("area") == Some(&"1".to_string()))
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "PointPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(PointPredicate) }
}

pub struct CrossPredicate;
impl Predicate for CrossPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| {
                if let Some(mask) = n.attributes.get("shape_mask") {
                    let pixels: HashSet<(i32,i32)> = mask.split(';').filter_map(|s| {
                        let mut it = s.split(','); Some((it.next()?.parse().ok()?, it.next()?.parse().ok()?))
                    }).collect();
                    (0..5).all(|i| pixels.contains(&(i,2))) && (0..5).all(|i| pixels.contains(&(2,i))) && pixels.len() >= 5
                } else { false }
            }).map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "CrossPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(CrossPredicate) }
}

pub struct HolePredicate;
impl Predicate for HolePredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.edges.iter()
            .filter(|e| e.rel_type == "contains")
            .map(|e| {
                let inner_node = graph.nodes.iter().find(|n| n.id == e.to);
                let score = inner_node.and_then(|n| n.attributes.get("area").and_then(|v| v.parse::<f32>().ok())).unwrap_or(0.5);
                (e.to.clone(), score)
            }).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "HolePredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(HolePredicate) }
}

pub struct ConvexPredicate;
impl Predicate for ConvexPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        // Heurisztika: ha a shape_mask kitölt egy bounding box-ot, akkor konvex (téglalap)
        let rect = RectanglePredicate.evaluate(graph);
        if let PredicateResult::RankedList(list) = rect { PredicateResult::RankedList(list) } else { PredicateResult::Bool(false) }
    }
    fn name(&self) -> &str { "ConvexPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(ConvexPredicate) }
}

// ---------------------------------------------------------------------
// Kvantitatív predikátumok
// ---------------------------------------------------------------------
pub struct EqualAreaPredicate { pub reference: Box<dyn Predicate> }
impl Predicate for EqualAreaPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        let ref_areas: Vec<u64> = graph.nodes.iter()
            .filter(|n| ref_ids.contains(&n.id))
            .filter_map(|n| n.attributes.get("area").and_then(|v| v.parse::<u64>().ok())).collect();
        if ref_areas.is_empty() { return PredicateResult::Bool(false); }
        let target_area = ref_areas[0];
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| !ref_ids.contains(&n.id))
            .filter(|n| n.attributes.get("area").and_then(|v| v.parse::<u64>().ok()) == Some(target_area))
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "EqualAreaPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(EqualAreaPredicate { reference: self.reference.clone_box() }) }
}

// Hasonlóan lehetne EqualColor, EqualWidth stb., de most kihagyjuk a terjedelem miatt.

// ---------------------------------------------------------------------
// Logikai predikátumok (And, Not már van, hozzáadjuk az Or-t)
// ---------------------------------------------------------------------
pub struct OrPredicate { pub predicates: Vec<Box<dyn Predicate>> }
impl Predicate for OrPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let mut results = Vec::new();
        for p in &self.predicates {
            match p.evaluate(graph) {
                PredicateResult::Bool(true) => return PredicateResult::Bool(true),
                PredicateResult::RankedList(list) => results.extend(list),
                _ => {}
            }
        }
        if results.is_empty() { PredicateResult::Bool(false) } else {
            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            results.dedup_by_key(|(id, _)| id.clone());
            PredicateResult::RankedList(results)
        }
    }
    fn name(&self) -> &str { "OrPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(OrPredicate { predicates: self.predicates.iter().map(|p| p.clone_box()).collect() }) }
}

pub struct AndPredicate { pub predicates: Vec<Box<dyn Predicate>> }
impl Predicate for AndPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let mut result = PredicateResult::Bool(true);
        for p in &self.predicates {
            match p.evaluate(graph) {
                PredicateResult::Bool(false) => return PredicateResult::Bool(false),
                PredicateResult::RankedList(list) => {
                    result = match result {
                        PredicateResult::Bool(true) => PredicateResult::RankedList(list),
                        PredicateResult::RankedList(prev) => {
                            let intersection: Vec<_> = prev.into_iter()
                                .filter(|(id, _)| list.iter().any(|(i, _)| i == id))
                                .map(|(id, score)| {
                                    let other_score = list.iter().find(|(i, _)| i == &id).map(|(_, s)| *s).unwrap_or(0.0);
                                    (id, (score + other_score) / 2.0)
                                }).collect();
                            PredicateResult::RankedList(intersection)
                        }
                        _ => result,
                    };
                }
                _ => {}
            }
        }
        result
    }
    fn name(&self) -> &str { "AndPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(AndPredicate { predicates: self.predicates.iter().map(|p| p.clone_box()).collect() }) }
}

pub struct NotPredicate { pub predicate: Box<dyn Predicate> }
impl Predicate for NotPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        match self.predicate.evaluate(graph) {
            PredicateResult::Bool(b) => PredicateResult::Bool(!b),
            PredicateResult::RankedList(list) => {
                let all_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
                let excluded: Vec<String> = list.into_iter().map(|(id, _)| id).collect();
                let remaining: Vec<(String, f32)> = all_ids.into_iter()
                    .filter(|id| !excluded.contains(id)).map(|id| (id, 1.0)).collect();
                PredicateResult::RankedList(remaining)
            }
        }
    }
    fn name(&self) -> &str { "NotPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(NotPredicate { predicate: self.predicate.clone_box() }) }
}

// Relációs predikátumok (LeftOf, RightOf, Above, Below, Adjacent) már korábban implementálva, de újra kell definiálni a clone_box miatt.
// Bemásoljuk a korábbi implementációkat clone_box-szal.
pub struct LeftOfPredicate { pub reference: Box<dyn Predicate> }
impl Predicate for LeftOfPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        let ref_bbox_x: Vec<i64> = graph.nodes.iter().filter(|n| ref_ids.contains(&n.id))
            .filter_map(|n| n.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok())).collect();
        if ref_bbox_x.is_empty() { return PredicateResult::Bool(false); }
        let min_ref_x = *ref_bbox_x.iter().min().unwrap();
        let matching: Vec<(String, f32)> = graph.nodes.iter().filter(|n| !ref_ids.contains(&n.id))
            .filter_map(|n| {
                let x: i64 = n.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
                let w: i64 = n.attributes.get("bbox_w").and_then(|v| v.parse().ok())?;
                if x + w <= min_ref_x { Some((n.id.clone(), 1.0)) } else { None }
            }).collect();
        PredicateResult::RankedList(matching)
    }
    fn name(&self) -> &str { "LeftOfPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(LeftOfPredicate { reference: self.reference.clone_box() }) }
}

pub struct RightOfPredicate { pub reference: Box<dyn Predicate> }
impl Predicate for RightOfPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        let ref_bbox_right: Vec<i64> = graph.nodes.iter().filter(|n| ref_ids.contains(&n.id))
            .filter_map(|n| { let x = n.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok())?; let w = n.attributes.get("bbox_w").and_then(|v| v.parse::<i64>().ok())?; Some(x + w) }).collect();
        if ref_bbox_right.is_empty() { return PredicateResult::Bool(false); }
        let max_ref_right = *ref_bbox_right.iter().max().unwrap();
        let matching: Vec<(String, f32)> = graph.nodes.iter().filter(|n| !ref_ids.contains(&n.id))
            .filter_map(|n| { let x = n.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok())?; if x >= max_ref_right { Some((n.id.clone(), 1.0)) } else { None } }).collect();
        PredicateResult::RankedList(matching)
    }
    fn name(&self) -> &str { "RightOfPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(RightOfPredicate { reference: self.reference.clone_box() }) }
}

pub struct AbovePredicate { pub reference: Box<dyn Predicate> }
impl Predicate for AbovePredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        let ref_bbox_top: Vec<i64> = graph.nodes.iter().filter(|n| ref_ids.contains(&n.id))
            .filter_map(|n| { n.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()) }).collect();
        if ref_bbox_top.is_empty() { return PredicateResult::Bool(false); }
        let min_ref_y = *ref_bbox_top.iter().min().unwrap();
        let matching: Vec<(String, f32)> = graph.nodes.iter().filter(|n| !ref_ids.contains(&n.id))
            .filter_map(|n| {
                let y = n.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok())?;
                let h = n.attributes.get("bbox_h").and_then(|v| v.parse::<i64>().ok())?;
                if y + h <= min_ref_y { Some((n.id.clone(), 1.0)) } else { None }
            }).collect();
        PredicateResult::RankedList(matching)
    }
    fn name(&self) -> &str { "AbovePredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(AbovePredicate { reference: self.reference.clone_box() }) }
}

pub struct BelowPredicate { pub reference: Box<dyn Predicate> }
impl Predicate for BelowPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        let ref_bbox_bottom: Vec<i64> = graph.nodes.iter().filter(|n| ref_ids.contains(&n.id))
            .filter_map(|n| { let y = n.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok())?; let h = n.attributes.get("bbox_h").and_then(|v| v.parse::<i64>().ok())?; Some(y + h) }).collect();
        if ref_bbox_bottom.is_empty() { return PredicateResult::Bool(false); }
        let max_ref_bottom = *ref_bbox_bottom.iter().max().unwrap();
        let matching: Vec<(String, f32)> = graph.nodes.iter().filter(|n| !ref_ids.contains(&n.id))
            .filter_map(|n| { let y = n.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok())?; if y >= max_ref_bottom { Some((n.id.clone(), 1.0)) } else { None } }).collect();
        PredicateResult::RankedList(matching)
    }
    fn name(&self) -> &str { "BelowPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(BelowPredicate { reference: self.reference.clone_box() }) }
}

pub struct AdjacentPredicate { pub reference: Box<dyn Predicate> }
impl Predicate for AdjacentPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        let matching: Vec<(String, f32)> = graph.nodes.iter().filter(|n| !ref_ids.contains(&n.id))
            .filter(|n| graph.edges.iter().any(|e| (e.from == n.id && ref_ids.contains(&e.to) && e.rel_type == "touches") || (e.to == n.id && ref_ids.contains(&e.from) && e.rel_type == "touches")))
            .map(|n| (n.id.clone(), 1.0)).collect();
        PredicateResult::RankedList(matching)
    }
    fn name(&self) -> &str { "AdjacentPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(AdjacentPredicate { reference: self.reference.clone_box() }) }
}

// --- Connected ---
pub struct ConnectedPredicate { pub reference: Box<dyn Predicate> }

impl Predicate for ConnectedPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        
        // BFS a touches éleken keresztül
        let mut visited = std::collections::HashSet::new();
        let mut queue: Vec<String> = ref_ids.clone();
        for id in &ref_ids { visited.insert(id.clone()); }
        
        while let Some(current) = queue.pop() {
            for edge in &graph.edges {
                if edge.rel_type == "touches" {
                    let neighbor = if edge.from == current { &edge.to } else if edge.to == current { &edge.from } else { continue };
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        queue.push(neighbor.clone());
                    }
                }
            }
        }
        
        // Az összes node, ami a touches láncban van, de nem része a referenciának
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| !ref_ids.contains(&n.id) && visited.contains(&n.id))
            .map(|n| (n.id.clone(), 1.0))
            .collect();
        
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "ConnectedPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(ConnectedPredicate { reference: self.reference.clone_box() }) }
}

// --- Inside ---
pub struct InsidePredicate { pub reference: Box<dyn Predicate> }

impl Predicate for InsidePredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        
        let matching: Vec<(String, f32)> = graph.edges.iter()
            .filter(|e| e.rel_type == "contains" && ref_ids.contains(&e.from))
            .map(|e| (e.to.clone(), 1.0))
            .collect();
        
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "InsidePredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(InsidePredicate { reference: self.reference.clone_box() }) }
}

// --- Contains ---
pub struct ContainsPredicate { pub reference: Box<dyn Predicate> }

impl Predicate for ContainsPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        
        let matching: Vec<(String, f32)> = graph.edges.iter()
            .filter(|e| e.rel_type == "contains" && ref_ids.contains(&e.to))
            .map(|e| (e.from.clone(), 1.0))
            .collect();
        
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "ContainsPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(ContainsPredicate { reference: self.reference.clone_box() }) }
}

// --- Nearest ---
pub struct NearestPredicate { pub reference: Box<dyn Predicate> }

impl Predicate for NearestPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        
        let ref_centers: Vec<(f64, f64)> = graph.nodes.iter()
            .filter(|n| ref_ids.contains(&n.id))
            .filter_map(|n| {
                let x: f64 = n.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
                let y: f64 = n.attributes.get("bbox_y").and_then(|v| v.parse().ok())?;
                let w: f64 = n.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                let h: f64 = n.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                Some((x + w / 2.0, y + h / 2.0))
            }).collect();
        if ref_centers.is_empty() { return PredicateResult::Bool(false); }
        let ref_cx = ref_centers.iter().map(|(x, _)| x).sum::<f64>() / ref_centers.len() as f64;
        let ref_cy = ref_centers.iter().map(|(_, y)| y).sum::<f64>() / ref_centers.len() as f64;
        
        let mut scored: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| !ref_ids.contains(&n.id))
            .filter_map(|n| {
                let x: f64 = n.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
                let y: f64 = n.attributes.get("bbox_y").and_then(|v| v.parse().ok())?;
                let w: f64 = n.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                let h: f64 = n.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                let cx = x + w / 2.0;
                let cy = y + h / 2.0;
                let dist = ((cx - ref_cx).powi(2) + (cy - ref_cy).powi(2)).sqrt();
                Some((n.id.clone(), 1.0 / (1.0 + dist as f32)))
            }).collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        if scored.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(scored) }
    }
    fn name(&self) -> &str { "NearestPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(NearestPredicate { reference: self.reference.clone_box() }) }
}

// --- Farthest ---
pub struct FarthestPredicate { pub reference: Box<dyn Predicate> }

impl Predicate for FarthestPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        
        let ref_centers: Vec<(f64, f64)> = graph.nodes.iter()
            .filter(|n| ref_ids.contains(&n.id))
            .filter_map(|n| {
                let x: f64 = n.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
                let y: f64 = n.attributes.get("bbox_y").and_then(|v| v.parse().ok())?;
                let w: f64 = n.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                let h: f64 = n.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                Some((x + w / 2.0, y + h / 2.0))
            }).collect();
        if ref_centers.is_empty() { return PredicateResult::Bool(false); }
        let ref_cx = ref_centers.iter().map(|(x, _)| x).sum::<f64>() / ref_centers.len() as f64;
        let ref_cy = ref_centers.iter().map(|(_, y)| y).sum::<f64>() / ref_centers.len() as f64;
        
        let mut scored: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| !ref_ids.contains(&n.id))
            .filter_map(|n| {
                let x: f64 = n.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
                let y: f64 = n.attributes.get("bbox_y").and_then(|v| v.parse().ok())?;
                let w: f64 = n.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                let h: f64 = n.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1.0);
                let cx = x + w / 2.0;
                let cy = y + h / 2.0;
                let dist = ((cx - ref_cx).powi(2) + (cy - ref_cy).powi(2)).sqrt();
                Some((n.id.clone(), dist as f32 / 100.0))
            }).collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        if scored.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(scored) }
    }
    fn name(&self) -> &str { "FarthestPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(FarthestPredicate { reference: self.reference.clone_box() }) }
}

// --- ObjectCount ---
pub struct ObjectCountPredicate { pub min: usize, pub max: usize }

impl Predicate for ObjectCountPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let count = graph.nodes.iter().filter(|n| n.node_type == "arc_object").count();
        PredicateResult::Bool(count >= self.min && count <= self.max)
    }
    fn name(&self) -> &str { "ObjectCountPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(ObjectCountPredicate { min: self.min, max: self.max }) }
}

// --- NeighbourCount ---
pub struct NeighbourCountPredicate { pub reference: Box<dyn Predicate>, pub min: usize, pub max: usize }

impl Predicate for NeighbourCountPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| ref_ids.contains(&n.id))
            .filter(|n| {
                let count = graph.edges.iter()
                    .filter(|e| (e.from == n.id || e.to == n.id) && e.rel_type == "touches")
                    .count();
                count >= self.min && count <= self.max
            })
            .map(|n| (n.id.clone(), 1.0))
            .collect();
        
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "NeighbourCountPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(NeighbourCountPredicate { reference: self.reference.clone_box(), min: self.min, max: self.max }) }
}

// --- Intersects ---
pub struct IntersectsPredicate { pub reference: Box<dyn Predicate> }

impl Predicate for IntersectsPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        let ref_rects: Vec<(i64,i64,i64,i64)> = graph.nodes.iter()
            .filter(|n| ref_ids.contains(&n.id))
            .filter_map(|n| {
                let x: i64 = n.attributes.get("bbox_x").and_then(|v| v.parse().ok())?;
                let y: i64 = n.attributes.get("bbox_y").and_then(|v| v.parse().ok())?;
                let w: i64 = n.attributes.get("bbox_w").and_then(|v| v.parse().ok()).unwrap_or(1);
                let h: i64 = n.attributes.get("bbox_h").and_then(|v| v.parse().ok()).unwrap_or(1);
                Some((x, y, w, h))
            }).collect();
        
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| !ref_ids.contains(&n.id))
            .filter(|n| {
                if let (Some(x), Some(y), Some(w), Some(h)) = (
                    n.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()),
                    n.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()),
                    n.attributes.get("bbox_w").and_then(|v| v.parse::<i64>().ok()),
                    n.attributes.get("bbox_h").and_then(|v| v.parse::<i64>().ok()),
                ) {
                    ref_rects.iter().any(|(rx, ry, rw, rh)| {
                        x < *rx + *rw && x + w > *rx && y < *ry + *rh && y + h > *ry
                    })
                } else { false }
            })
            .map(|n| (n.id.clone(), 1.0))
            .collect();
        
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "IntersectsPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(IntersectsPredicate { reference: self.reference.clone_box() }) }
}

// --- Concave ---
pub struct ConcavePredicate;
impl Predicate for ConcavePredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        // Ha van Hole, akkor konkáv
        let hole_result = HolePredicate.evaluate(graph);
        if let PredicateResult::RankedList(list) = hole_result {
            if !list.is_empty() {
                return PredicateResult::RankedList(list);
            }
        }
        // Egyébként ha nem Rectangle, akkor is konkávnak tekintjük
        let not_rect = NotPredicate { predicate: Box::new(RectanglePredicate) };
        not_rect.evaluate(graph)
    }
    fn name(&self) -> &str { "ConcavePredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(ConcavePredicate) }
}

// --- Filled ---
pub struct FilledPredicate;
impl Predicate for FilledPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let hole = HolePredicate.evaluate(graph);
        match hole {
            PredicateResult::Bool(false) => PredicateResult::Bool(true),
            PredicateResult::RankedList(list) if list.is_empty() => PredicateResult::Bool(true),
            _ => PredicateResult::Bool(false),
        }
    }
    fn name(&self) -> &str { "FilledPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(FilledPredicate) }
}

// --- Hollow ---
pub struct HollowPredicate;
impl Predicate for HollowPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        HolePredicate.evaluate(graph)
    }
    fn name(&self) -> &str { "HollowPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(HollowPredicate) }
}

// --- EqualShape ---
pub struct EqualShapePredicate { pub reference: Box<dyn Predicate> }

impl Predicate for EqualShapePredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        let ref_masks: Vec<String> = graph.nodes.iter()
            .filter(|n| ref_ids.contains(&n.id))
            .filter_map(|n| n.attributes.get("shape_mask").cloned())
            .collect();
        if ref_masks.is_empty() { return PredicateResult::Bool(false); }
        let target = &ref_masks[0];
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| !ref_ids.contains(&n.id) && n.attributes.get("shape_mask") == Some(target))
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "EqualShapePredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(EqualShapePredicate { reference: self.reference.clone_box() }) }
}

// --- BoundingBox ---
pub struct BoundingBoxPredicate { pub x: Option<i64>, pub y: Option<i64>, pub w: Option<u64>, pub h: Option<u64> }
impl Predicate for BoundingBoxPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| {
                self.x.map_or(true, |v| n.attributes.get("bbox_x").and_then(|s| s.parse::<i64>().ok()) == Some(v)) &&
                self.y.map_or(true, |v| n.attributes.get("bbox_y").and_then(|s| s.parse::<i64>().ok()) == Some(v)) &&
                self.w.map_or(true, |v| n.attributes.get("bbox_w").and_then(|s| s.parse::<u64>().ok()) == Some(v)) &&
                self.h.map_or(true, |v| n.attributes.get("bbox_h").and_then(|s| s.parse::<u64>().ok()) == Some(v))
            })
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "BoundingBoxPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(BoundingBoxPredicate { x: self.x, y: self.y, w: self.w, h: self.h }) }
}

// --- XOR ---
pub struct XorPredicate { pub a: Box<dyn Predicate>, pub b: Box<dyn Predicate> }
impl Predicate for XorPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let ra = self.a.evaluate(graph);
        let rb = self.b.evaluate(graph);
        match (ra, rb) {
            (PredicateResult::Bool(a), PredicateResult::Bool(b)) => PredicateResult::Bool(a ^ b),
            (PredicateResult::RankedList(list_a), PredicateResult::RankedList(list_b)) => {
                let ids_a: Vec<String> = list_a.iter().map(|(id, _)| id.clone()).collect();
                let ids_b: Vec<String> = list_b.iter().map(|(id, _)| id.clone()).collect();
                let unique: Vec<(String, f32)> = list_a.into_iter()
                    .filter(|(id, _)| !ids_b.contains(id))
                    .chain(list_b.into_iter().filter(|(id, _)| !ids_a.contains(id)))
                    .collect();
                if unique.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(unique) }
            }
            (PredicateResult::RankedList(list), PredicateResult::Bool(b)) => {
                if b { PredicateResult::RankedList(list) } else { PredicateResult::Bool(false) }
            }
            (PredicateResult::Bool(a), PredicateResult::RankedList(list)) => {
                if a { PredicateResult::RankedList(list) } else { PredicateResult::Bool(false) }
            }
        }
    }
    fn name(&self) -> &str { "XorPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(XorPredicate { a: self.a.clone_box(), b: self.b.clone_box() }) }
}

// --- MirrorSymmetric (biztosra menő újradefiniálás) ---
pub struct MirrorSymmetricPredicate;
impl Predicate for MirrorSymmetricPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let objects: Vec<_> = graph.nodes.iter().filter(|n| n.node_type == "arc_object").collect();
        for i in 0..objects.len() {
            for j in (i+1)..objects.len() {
                let a = &objects[i]; let b = &objects[j];
                if a.attributes.get("shape_mask") == b.attributes.get("shape_mask") &&
                   a.attributes.get("color") == b.attributes.get("color") {
                    if let (Some(ax), Some(ay), Some(bx), Some(by)) = (
                        a.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()),
                        a.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()),
                        b.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()),
                        b.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()),
                    ) {
                        if ax == bx || ay == by {
                            return PredicateResult::RankedList(vec![(a.id.clone(), 1.0), (b.id.clone(), 1.0)]);
                        }
                    }
                }
            }
        }
        PredicateResult::Bool(false)
    }
    fn name(&self) -> &str { "MirrorSymmetricPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(MirrorSymmetricPredicate) }
}

// --- RotationalSymmetry ---
pub struct RotationalSymmetryPredicate;
impl Predicate for RotationalSymmetryPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let objects: Vec<_> = graph.nodes.iter().filter(|n| n.node_type == "arc_object").collect();
        for i in 0..objects.len() {
            for j in (i+1)..objects.len() {
                let a = &objects[i]; let b = &objects[j];
                if a.attributes.get("shape_mask") == b.attributes.get("shape_mask") &&
                   a.attributes.get("color") == b.attributes.get("color") &&
                   a.attributes.get("area") == b.attributes.get("area") {
                    return PredicateResult::RankedList(vec![(a.id.clone(), 1.0), (b.id.clone(), 1.0)]);
                }
            }
        }
        PredicateResult::Bool(false)
    }
    fn name(&self) -> &str { "RotationalSymmetryPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(RotationalSymmetryPredicate) }
}

// --- EqualColor ---
pub struct EqualColorPredicate { pub reference: Box<dyn Predicate> }
impl Predicate for EqualColorPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        let ref_colors: Vec<String> = graph.nodes.iter()
            .filter(|n| ref_ids.contains(&n.id))
            .filter_map(|n| n.attributes.get("color").cloned())
            .collect();
        if ref_colors.is_empty() { return PredicateResult::Bool(false); }
        let target = &ref_colors[0];
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| !ref_ids.contains(&n.id) && n.attributes.get("color") == Some(target))
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "EqualColorPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(EqualColorPredicate { reference: self.reference.clone_box() }) }
}

// --- EqualWidth ---
pub struct EqualWidthPredicate { pub reference: Box<dyn Predicate> }
impl Predicate for EqualWidthPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        let ref_widths: Vec<String> = graph.nodes.iter()
            .filter(|n| ref_ids.contains(&n.id))
            .filter_map(|n| n.attributes.get("bbox_w").cloned())
            .collect();
        if ref_widths.is_empty() { return PredicateResult::Bool(false); }
        let target = &ref_widths[0];
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| !ref_ids.contains(&n.id) && n.attributes.get("bbox_w") == Some(target))
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "EqualWidthPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(EqualWidthPredicate { reference: self.reference.clone_box() }) }
}

// --- EqualHeight ---
pub struct EqualHeightPredicate { pub reference: Box<dyn Predicate> }
impl Predicate for EqualHeightPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        let ref_heights: Vec<String> = graph.nodes.iter()
            .filter(|n| ref_ids.contains(&n.id))
            .filter_map(|n| n.attributes.get("bbox_h").cloned())
            .collect();
        if ref_heights.is_empty() { return PredicateResult::Bool(false); }
        let target = &ref_heights[0];
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| !ref_ids.contains(&n.id) && n.attributes.get("bbox_h") == Some(target))
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "EqualHeightPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(EqualHeightPredicate { reference: self.reference.clone_box() }) }
}

// --- EqualNeighbourCount ---
pub struct EqualNeighbourCountPredicate { pub reference: Box<dyn Predicate> }
impl Predicate for EqualNeighbourCountPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        let refs = self.reference.evaluate(graph).as_ranked_list();
        if refs.is_empty() { return PredicateResult::Bool(false); }
        let ref_ids: Vec<String> = refs.into_iter().map(|(id, _)| id).collect();
        let ref_counts: Vec<usize> = graph.nodes.iter()
            .filter(|n| ref_ids.contains(&n.id))
            .map(|n| graph.edges.iter().filter(|e| (e.from == n.id || e.to == n.id) && e.rel_type == "touches").count())
            .collect();
        if ref_counts.is_empty() { return PredicateResult::Bool(false); }
        let target = ref_counts[0];
        let matching: Vec<(String, f32)> = graph.nodes.iter()
            .filter(|n| !ref_ids.contains(&n.id))
            .filter(|n| {
                let count = graph.edges.iter().filter(|e| (e.from == n.id || e.to == n.id) && e.rel_type == "touches").count();
                count == target
            })
            .map(|n| (n.id.clone(), 1.0)).collect();
        if matching.is_empty() { PredicateResult::Bool(false) } else { PredicateResult::RankedList(matching) }
    }
    fn name(&self) -> &str { "EqualNeighbourCountPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(EqualNeighbourCountPredicate { reference: self.reference.clone_box() }) }
}

// --- IfPredicate (már létezik, de biztosra) ---
pub struct IfPredicate { pub condition: Box<dyn Predicate>, pub then_branch: Box<dyn Predicate>, pub else_branch: Box<dyn Predicate> }
impl Predicate for IfPredicate {
    fn evaluate(&self, graph: &KernelStructureGraph) -> PredicateResult {
        if self.condition.evaluate(graph).is_true() {
            self.then_branch.evaluate(graph)
        } else {
            self.else_branch.evaluate(graph)
        }
    }
    fn name(&self) -> &str { "IfPredicate" }
    fn clone_box(&self) -> Box<dyn Predicate> { Box::new(IfPredicate {
        condition: self.condition.clone_box(),
        then_branch: self.then_branch.clone_box(),
        else_branch: self.else_branch.clone_box(),
    }) }
}
