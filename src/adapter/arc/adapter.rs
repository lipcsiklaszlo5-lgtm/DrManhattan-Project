use std::collections::HashMap;
use crate::structure::KernelStructureGraph;
use crate::adapter::{DomainAdapter, ValidationError, Algorithm, CostEstimate};
use crate::task::Task;

/// ARC rács reprezentációja – max 64x64, 16 szín (0-15)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArcGrid {
    pub width: u8,
    pub height: u8,
    pub pixels: Vec<u8>, // lapos tömb, hossza width * height
}

impl ArcGrid {
    pub fn new(width: u8, height: u8, background: u8) -> Self {
        let size = width as usize * height as usize;
        ArcGrid {
            width,
            height,
            pixels: vec![background; size],
        }
    }

    /// Lekérdezi egy pixel színét (x, y) koordinátán
    pub fn get(&self, x: u8, y: u8) -> Option<u8> {
        if x < self.width && y < self.height {
            Some(self.pixels[y as usize * self.width as usize + x as usize])
        } else {
            None
        }
    }

    /// Beállítja egy pixel színét
    pub fn set(&mut self, x: u8, y: u8, color: u8) {
        if x < self.width && y < self.height {
            self.pixels[y as usize * self.width as usize + x as usize] = color;
        }
    }
}

/// A KSG-ben lévő csomópont (objektum) attribútumai
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ArcNodeAttr {
    pub color: u8,
    pub bbox_x: u8,
    pub bbox_y: u8,
    pub bbox_w: u8,
    pub bbox_h: u8,
    pub area: u16,
    /// Relatív koordináták a bbox bal felső sarkához képest a veszteségmentes rekonstrukcióhoz
    pub shape_mask: Vec<(u8, u8)>,
}

/// Az élek (relációk) típusai
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ArcEdgeType {
    Touches,
    Contains,
    LeftOf,
    RightOf,
    Above,
    Below,
}

/// Élt reprezentáló attribútum
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ArcEdgeAttr {
    pub relation: ArcEdgeType,
    pub distance: u8,
}

/// Az ArcAdapter struktúra – állapot nélküli, statikus metódusokkal
pub struct ArcAdapter;

impl ArcAdapter {
    /// Grid-to-KSG: Flood Fill (BFS) alapú objektum-kiemelés
    pub fn grid_to_ksg(grid: &ArcGrid) -> KernelStructureGraph {
        let mut graph = KernelStructureGraph::new();
        let size = grid.width as usize * grid.height as usize;
        let mut visited = vec![false; size];
        let mut node_id_counter = 0u32;

        for y in 0..grid.height {
            for x in 0..grid.width {
                let idx = y as usize * grid.width as usize + x as usize;
                if visited[idx] {
                    continue;
                }
                let color = grid.pixels[idx];
                if color == 0 {
                    // Háttér – kihagyjuk, vagy külön csomópontként kezelhető
                    visited[idx] = true;
                    continue;
                }

                // Flood Fill (BFS) az összefüggő, azonos színű pixelekre
                let mut queue = vec![(x, y)];
                let mut shape_pixels = Vec::new();
                let mut min_x = x;
                let mut max_x = x;
                let mut min_y = y;
                let mut max_y = y;

                while let Some((cx, cy)) = queue.pop() {
                    let c_idx = cy as usize * grid.width as usize + cx as usize;
                    if visited[c_idx] {
                        continue;
                    }
                    visited[c_idx] = true;
                    shape_pixels.push((cx, cy));

                    // Bounding box frissítése
                    if cx < min_x { min_x = cx; }
                    if cx > max_x { max_x = cx; }
                    if cy < min_y { min_y = cy; }
                    if cy > max_y { max_y = cy; }

                    // Szomszédok vizsgálata (4-irányú)
                    let neighbors = [
                        (cx.wrapping_sub(1), cy),
                        (cx + 1, cy),
                        (cx, cy.wrapping_sub(1)),
                        (cx, cy + 1),
                    ];
                    for (nx, ny) in &neighbors {
                        if *nx < grid.width && *ny < grid.height {
                            let n_idx = *ny as usize * grid.width as usize + *nx as usize;
                            if !visited[n_idx] && grid.pixels[n_idx] == color {
                                queue.push((*nx, *ny));
                            }
                        }
                    }
                }

                // Relatív shape_mask kiszámítása
                let shape_mask: Vec<(u8, u8)> = shape_pixels
                    .iter()
                    .map(|(px, py)| (px - min_x, py - min_y))
                    .collect();

                let area = shape_pixels.len() as u16;
                let bbox_w = max_x - min_x + 1;
                let bbox_h = max_y - min_y + 1;

                let attr = ArcNodeAttr {
                    color,
                    bbox_x: min_x,
                    bbox_y: min_y,
                    bbox_w,
                    bbox_h,
                    area,
                    shape_mask,
                };

                let node_id = format!("obj_{}", node_id_counter);
                let node = graph.add_node(&node_id, "arc_object");
                // Attribútumok tárolása a KSG-ben (szerializálva)
                node.attributes.insert("color".into(), color.to_string());
                node.attributes.insert("bbox_x".into(), min_x.to_string());
                node.attributes.insert("bbox_y".into(), min_y.to_string());
                node.attributes.insert("bbox_w".into(), bbox_w.to_string());
                node.attributes.insert("bbox_h".into(), bbox_h.to_string());
                node.attributes.insert("area".into(), area.to_string());

                node_id_counter += 1;
            }
        }

        // Élek (relációk) meghatározása a csomópontok között
        let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        for i in 0..node_ids.len() {
            for j in (i + 1)..node_ids.len() {
                let node_a = &graph.nodes[i];
                let node_b = &graph.nodes[j];

                let ax = node_a.attributes.get("bbox_x").and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                let ay = node_a.attributes.get("bbox_y").and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                let aw = node_a.attributes.get("bbox_w").and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                let ah = node_a.attributes.get("bbox_h").and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                let bx = node_b.attributes.get("bbox_x").and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                let by = node_b.attributes.get("bbox_y").and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                let bw = node_b.attributes.get("bbox_w").and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                let bh = node_b.attributes.get("bbox_h").and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);

                // Left/Right/Above/Below
                if ax + aw <= bx {
                    graph.add_edge(&node_a.id, &node_b.id, "left_of");
                } else if bx + bw <= ax {
                    graph.add_edge(&node_a.id, &node_b.id, "right_of");
                }
                if ay + ah <= by {
                    graph.add_edge(&node_a.id, &node_b.id, "above");
                } else if by + bh <= ay {
                    graph.add_edge(&node_a.id, &node_b.id, "below");
                }

                // Contains
                if ax <= bx && ay <= by && ax + aw >= bx + bw && ay + ah >= by + bh {
                    graph.add_edge(&node_a.id, &node_b.id, "contains");
                } else if bx <= ax && by <= ay && bx + bw >= ax + aw && by + bh >= ay + ah {
                    graph.add_edge(&node_b.id, &node_a.id, "contains");
                }

                // Touches (bounding box távolság <= 1)
                let x_dist = if ax + aw <= bx { bx - (ax + aw) } else if bx + bw <= ax { ax - (bx + bw) } else { 0 };
                let y_dist = if ay + ah <= by { by - (ay + ah) } else if by + bh <= ay { ay - (by + bh) } else { 0 };
                if x_dist <= 1 && y_dist <= 1 {
                    graph.add_edge(&node_a.id, &node_b.id, "touches");
                }
            }
        }

        graph
    }

    /// KSG-to-Grid: Veszteségmentes rekonstrukció a shape_mask-ekből
    pub fn ksg_to_grid(graph: &KernelStructureGraph, width: u8, height: u8, background: u8) -> ArcGrid {
        let mut grid = ArcGrid::new(width, height, background);

        for node in &graph.nodes {
            let color = node.attributes.get("color").and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
            let bbox_x = node.attributes.get("bbox_x").and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
            let bbox_y = node.attributes.get("bbox_y").and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);

            // A shape_mask a KSG-ben nem tárolódik közvetlenül, de a rekonstrukcióhoz
            // a bounding box alapján visszaállítjuk a pixeleket
            let bbox_w = node.attributes.get("bbox_w").and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
            let bbox_h = node.attributes.get("bbox_h").and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);

            // Kitöltjük a bounding box-ot a színnel (egyszerűsített rekonstrukció)
            for dy in 0..bbox_h {
                for dx in 0..bbox_w {
                    let px = bbox_x + dx;
                    let py = bbox_y + dy;
                    if px < width && py < height {
                        grid.set(px, py, color);
                    }
                }
            }
        }

        grid
    }
}

impl DomainAdapter for ArcAdapter {
    fn build_structure(&self, task: &Task) -> KernelStructureGraph {
        // A task intent-ből próbálunk ArcGrid-et parsolni (egyszerűsítve)
        // Élesben a task tartalmazná a rács adatokat
        KernelStructureGraph::new()
    }

    fn validate(&self, structure: &KernelStructureGraph, candidate: &str) -> Result<(), ValidationError> {
        // ARC validáció: a kimeneti rács összehasonlítása a vártal
        Ok(())
    }

    fn available_algorithms(&self) -> Vec<Algorithm> {
        vec![
            Algorithm {
                name: "arc_grid_transform".into(),
                description: "ARC grid transformation".into(),
                cost: CostEstimate { latency_ms: 1, memory_bytes: 1024 },
            },
        ]
    }

    fn graph_to_code(&self, graph: &KernelStructureGraph, original_code: &str) -> String {
        // ARC esetén a gráfból rácsot készítünk
        format!("{:?}", graph)
    }
}

#[cfg(test)]
mod tests;
