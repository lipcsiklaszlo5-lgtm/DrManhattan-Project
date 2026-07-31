use crate::structure::KernelStructureGraph;
use crate::adapter::arc::adapter::ArcGrid;
use std::collections::{HashMap, VecDeque};

pub struct RepresentationFactory;

impl RepresentationFactory {
    pub fn color_graph(grid: &ArcGrid) -> KernelStructureGraph {
        crate::adapter::arc::ArcAdapter::grid_to_ksg(grid)
    }

    pub fn connected_components(grid: &ArcGrid) -> KernelStructureGraph {
        let mut graph = KernelStructureGraph::new();
        let mut visited = vec![false; grid.pixels.len()];
        let width = grid.width as usize;

        for y in 0..grid.height as usize {
            for x in 0..width {
                let idx = y * width + x;
                if visited[idx] || grid.pixels[idx] == 0 { visited[idx] = true; continue; }

                let mut shape_pixels = Vec::new();
                let mut queue = VecDeque::new();
                queue.push_back((x, y));
                let (mut min_x, mut max_x) = (x, x);
                let (mut min_y, mut max_y) = (y, y);

                while let Some((cx, cy)) = queue.pop_front() {
                    let c_idx = cy * width + cx;
                    if visited[c_idx] { continue; }
                    visited[c_idx] = true;
                    shape_pixels.push((cx, cy));
                    min_x = min_x.min(cx); max_x = max_x.max(cx);
                    min_y = min_y.min(cy); max_y = max_y.max(cy);

                    for (nx, ny) in [(cx.wrapping_sub(1), cy), (cx+1, cy), (cx, cy.wrapping_sub(1)), (cx, cy+1)] {
                        if nx < width && ny < grid.height as usize {
                            let n_idx = ny * width + nx;
                            if !visited[n_idx] && grid.pixels[n_idx] != 0 { queue.push_back((nx, ny)); }
                        }
                    }
                }

                let node_id = format!("comp_{}", graph.nodes.len());
                let mut attrs = HashMap::new();
                attrs.insert("bbox_x".into(), min_x.to_string());
                attrs.insert("bbox_y".into(), min_y.to_string());
                attrs.insert("bbox_w".into(), ((max_x-min_x+1) as u8).to_string());
                attrs.insert("bbox_h".into(), ((max_y-min_y+1) as u8).to_string());
                attrs.insert("area".into(), shape_pixels.len().to_string());
                graph.add_node(&node_id, "connected_component");
                if let Some(node) = graph.nodes.last_mut() { node.attributes = attrs; }
            }
        }
        Self::add_spatial_relations(&mut graph);
        graph
    }

    pub fn symmetry_graph(grid: &ArcGrid) -> KernelStructureGraph {
        let mut graph = KernelStructureGraph::new();
        let w = grid.width as usize;
        let h = grid.height as usize;
        let mut h_sym = true;
        for y in 0..h { for x in 0..w/2 { if grid.pixels[y*w+x] != grid.pixels[y*w+(w-1-x)] { h_sym = false; break; } } if !h_sym { break; } }
        let mut v_sym = true;
        for x in 0..w { for y in 0..h/2 { if grid.pixels[y*w+x] != grid.pixels[(h-1-y)*w+x] { v_sym = false; break; } } if !v_sym { break; } }
        if h_sym { let mut attrs = HashMap::new(); attrs.insert("axis".into(), "vertical".into()); attrs.insert("position".into(), (w/2).to_string()); graph.add_node("sym_h", "symmetry_axis"); if let Some(n) = graph.nodes.last_mut() { n.attributes = attrs; } }
        if v_sym { let mut attrs = HashMap::new(); attrs.insert("axis".into(), "horizontal".into()); attrs.insert("position".into(), (h/2).to_string()); graph.add_node("sym_v", "symmetry_axis"); if let Some(n) = graph.nodes.last_mut() { n.attributes = attrs; } }
        if graph.nodes.is_empty() { graph.add_node("no_symmetry", "symmetry_axis"); }
        graph
    }

    pub fn topology_graph(grid: &ArcGrid) -> KernelStructureGraph {
        let mut graph = Self::color_graph(grid);
        let nodes = graph.nodes.clone();
        for i in 0..nodes.len() {
            for j in i+1..nodes.len() {
                let (a, b) = (&nodes[i], &nodes[j]);
                if let (Some(ax), Some(ay), Some(aw), Some(ah), Some(bx), Some(by), Some(bw), Some(bh)) = (
                    a.attributes.get("bbox_x").and_then(|v| v.parse::<u64>().ok()),
                    a.attributes.get("bbox_y").and_then(|v| v.parse::<u64>().ok()),
                    a.attributes.get("bbox_w").and_then(|v| v.parse::<u64>().ok()),
                    a.attributes.get("bbox_h").and_then(|v| v.parse::<u64>().ok()),
                    b.attributes.get("bbox_x").and_then(|v| v.parse::<u64>().ok()),
                    b.attributes.get("bbox_y").and_then(|v| v.parse::<u64>().ok()),
                    b.attributes.get("bbox_w").and_then(|v| v.parse::<u64>().ok()),
                    b.attributes.get("bbox_h").and_then(|v| v.parse::<u64>().ok()),
                ) {
                    let (ar, abot, br, bbot) = (ax+aw, ay+ah, bx+bw, by+bh);
                    if ax<=bx && ar>=br && ay<=by && abot>=bbot { graph.add_edge(&a.id, &b.id, "contains"); }
                    else if bx<=ax && br>=ar && by<=ay && bbot>=abot { graph.add_edge(&b.id, &a.id, "contains"); }
                    let dx = if ar < bx { bx - ar } else if br < ax { ax - br } else { 0 };
                    let dy = if abot < by { by - abot } else if bbot < ay { ay - bbot } else { 0 };
                    if dx <= 1 && dy <= 1 && (dx > 0 || dy > 0) { graph.add_edge(&a.id, &b.id, "adjacent"); }
                }
            }
        }
        graph
    }

    pub fn all_representations(grid: &ArcGrid) -> Vec<(String, KernelStructureGraph)> {
        vec![
            ("color".into(), Self::color_graph(grid)),
            ("connected_components".into(), Self::connected_components(grid)),
            ("symmetry".into(), Self::symmetry_graph(grid)),
            ("topology".into(), Self::topology_graph(grid)),
        ]
    }

    fn add_spatial_relations(graph: &mut KernelStructureGraph) {
        let nodes = graph.nodes.clone();
        for i in 0..nodes.len() {
            for j in i+1..nodes.len() {
                let (a, b) = (&nodes[i], &nodes[j]);
                if let (Some(ax), Some(ay), Some(aw), Some(ah), Some(bx), Some(by), Some(bw), Some(bh)) = (
                    a.attributes.get("bbox_x").and_then(|v| v.parse::<u64>().ok()),
                    a.attributes.get("bbox_y").and_then(|v| v.parse::<u64>().ok()),
                    a.attributes.get("bbox_w").and_then(|v| v.parse::<u64>().ok()),
                    a.attributes.get("bbox_h").and_then(|v| v.parse::<u64>().ok()),
                    b.attributes.get("bbox_x").and_then(|v| v.parse::<u64>().ok()),
                    b.attributes.get("bbox_y").and_then(|v| v.parse::<u64>().ok()),
                    b.attributes.get("bbox_w").and_then(|v| v.parse::<u64>().ok()),
                    b.attributes.get("bbox_h").and_then(|v| v.parse::<u64>().ok()),
                ) {
                    let (ar, abot, br, bbot) = (ax+aw, ay+ah, bx+bw, by+bh);
                    if ar <= bx { graph.add_edge(&a.id, &b.id, "left_of"); }
                    else if br <= ax { graph.add_edge(&a.id, &b.id, "right_of"); }
                    if abot <= by { graph.add_edge(&a.id, &b.id, "above"); }
                    else if bbot <= ay { graph.add_edge(&a.id, &b.id, "below"); }
                }
            }
        }
    }
}
