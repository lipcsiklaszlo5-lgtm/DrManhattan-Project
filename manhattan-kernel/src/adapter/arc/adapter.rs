use serde::{Serialize, Deserialize};
use crate::structure::KernelStructureGraph;
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub struct ArcGrid {
    pub width: u8,
    pub height: u8,
    pub pixels: Vec<u8>,
}

impl ArcGrid {
    pub fn new(width: u8, height: u8, pixels: Vec<u8>) -> Self {
        assert_eq!(pixels.len(), (width as usize) * (height as usize));
        Self { width, height, pixels }
    }
}

pub struct ArcAdapter;

impl ArcAdapter {
    pub fn grid_to_ksg(grid: &ArcGrid) -> KernelStructureGraph {
        let mut graph = KernelStructureGraph::new();
        let mut visited = vec![false; grid.pixels.len()];
        let width = grid.width as usize;

        for y in 0..grid.height as usize {
            for x in 0..width {
                let idx = y * width + x;
                if visited[idx] || grid.pixels[idx] == 0 {
                    visited[idx] = true;
                    continue;
                }

                let color = grid.pixels[idx];
                let mut shape_pixels = Vec::new();
                let mut queue = VecDeque::new();
                queue.push_back((x, y));

                let (mut min_x, mut max_x) = (x, x);
                let (mut min_y, mut max_y) = (y, y);

                while let Some((cx, cy)) = queue.pop_front() {
                    let c_idx = cy * width + cx;
                    if visited[c_idx] {
                        continue;
                    }
                    visited[c_idx] = true;
                    shape_pixels.push((cx, cy));
                    min_x = min_x.min(cx);
                    max_x = max_x.max(cx);
                    min_y = min_y.min(cy);
                    max_y = max_y.max(cy);

                    let neighbors = [
                        (cx.wrapping_sub(1), cy),
                        (cx + 1, cy),
                        (cx, cy.wrapping_sub(1)),
                        (cx, cy + 1),
                    ];
                    for (nx, ny) in neighbors {
                        if nx < width && ny < grid.height as usize {
                            let n_idx = ny * width + nx;
                            if !visited[n_idx] && grid.pixels[n_idx] == color {
                                queue.push_back((nx, ny));
                            }
                        }
                    }
                }

                let node_id = format!("obj_{}", graph.nodes.len());
                let mut attrs = HashMap::new();
                attrs.insert("color".to_string(), color.to_string());
                attrs.insert("bbox_x".to_string(), min_x.to_string());
                attrs.insert("bbox_y".to_string(), min_y.to_string());
                attrs.insert("bbox_w".to_string(), ((max_x - min_x + 1) as u8).to_string());
                attrs.insert("bbox_h".to_string(), ((max_y - min_y + 1) as u8).to_string());
                attrs.insert("area".to_string(), shape_pixels.len().to_string());

                let mut shape_mask = Vec::new();
                for (px, py) in &shape_pixels {
                    shape_mask.push(format!("{},{}", px - min_x, py - min_y));
                }
                attrs.insert("shape_mask".to_string(), shape_mask.join(";"));

                graph.add_node(&node_id, "arc_object");
                if let Some(node) = graph.nodes.last_mut() {
                    node.attributes = attrs;
                }
            }
        }

        let nodes_clone = graph.nodes.clone();
        for i in 0..nodes_clone.len() {
            for j in (i + 1)..nodes_clone.len() {
                let a = &nodes_clone[i];
                let b = &nodes_clone[j];
                if let (Some(ax), Some(ay), Some(aw), Some(ah), Some(bx), Some(by), Some(bw), Some(bh)) = (
                    a.attributes.get("bbox_x").map(|v| v.parse::<u8>().unwrap_or(0)),
                    a.attributes.get("bbox_y").map(|v| v.parse::<u8>().unwrap_or(0)),
                    a.attributes.get("bbox_w").map(|v| v.parse::<u8>().unwrap_or(0)),
                    a.attributes.get("bbox_h").map(|v| v.parse::<u8>().unwrap_or(0)),
                    b.attributes.get("bbox_x").map(|v| v.parse::<u8>().unwrap_or(0)),
                    b.attributes.get("bbox_y").map(|v| v.parse::<u8>().unwrap_or(0)),
                    b.attributes.get("bbox_w").map(|v| v.parse::<u8>().unwrap_or(0)),
                    b.attributes.get("bbox_h").map(|v| v.parse::<u8>().unwrap_or(0)),
                ) {
                    let a_right = ax + aw;
                    let a_bottom = ay + ah;
                    let b_right = bx + bw;
                    let b_bottom = by + bh;

                    if a_right <= bx { graph.add_edge(&a.id, &b.id, "left_of"); }
                    else if b_right <= ax { graph.add_edge(&a.id, &b.id, "right_of"); }
                    if a_bottom <= by { graph.add_edge(&a.id, &b.id, "above"); }
                    else if b_bottom <= ay { graph.add_edge(&a.id, &b.id, "below"); }
                    if ax >= bx && a_right <= b_right && ay >= by && a_bottom <= b_bottom {
                        graph.add_edge(&b.id, &a.id, "contains");
                    } else if bx >= ax && b_right <= a_right && by >= ay && b_bottom <= a_bottom {
                        graph.add_edge(&a.id, &b.id, "contains");
                    }
                    if a_right >= bx && b_right >= ax && a_bottom >= by && b_bottom >= ay {
                        graph.add_edge(&a.id, &b.id, "touches");
                    }
                }
            }
        }

        graph
    }

    pub fn ksg_to_grid(graph: &KernelStructureGraph, width: u8, height: u8, bg_color: u8) -> ArcGrid {
        let mut pixels = vec![bg_color; (width as usize) * (height as usize)];
        for node in &graph.nodes {
            if let (Some(color_str), Some(x_str), Some(y_str), Some(mask_str)) = (
                node.attributes.get("color"),
                node.attributes.get("bbox_x"),
                node.attributes.get("bbox_y"),
                node.attributes.get("shape_mask"),
            ) {
                let color: u8 = color_str.parse().unwrap_or(bg_color);
                let bx: usize = x_str.parse().unwrap_or(0);
                let by: usize = y_str.parse().unwrap_or(0);
                for part in mask_str.split(';') {
                    let mut coords = part.split(',');
                    if let (Some(dx_str), Some(dy_str)) = (coords.next(), coords.next()) {
                        let dx: usize = dx_str.parse().unwrap_or(0);
                        let dy: usize = dy_str.parse().unwrap_or(0);
                        let px = bx + dx;
                        let py = by + dy;
                        if px < width as usize && py < height as usize {
                            pixels[py * width as usize + px] = color;
                        }
                    }
                }
            }
        }
        ArcGrid::new(width, height, pixels)
    }
}
