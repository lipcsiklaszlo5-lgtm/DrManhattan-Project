use crate::adapter::arc::adapter::ArcGrid;
use crate::adapter::arc::ArcAdapter;
use crate::structure::KernelStructureGraph;
use crate::sandbox::operators::Transformation;
use crate::abstraction::program::Program;
use crate::structure::SplitMix64;

pub struct SyntheticArcGenerator {
    seed: u64,
    max_objects: usize,
    max_operators: usize,
    grid_size: u8,
}

impl SyntheticArcGenerator {
    pub fn new(seed: u64, max_objects: usize, max_operators: usize, grid_size: u8) -> Self {
        Self { seed, max_objects, max_operators, grid_size }
    }

    fn random_grid(&self, state: &mut SplitMix64) -> ArcGrid {
        let mut pixels = vec![0u8; (self.grid_size as usize) * (self.grid_size as usize)];
        let num_objects = state.next_u64() as usize % self.max_objects + 1;
        for _ in 0..num_objects {
            let color = (state.next_u64() % 9 + 1) as u8;
            let w = (state.next_u64() % 3 + 1) as u8;
            let h = (state.next_u64() % 3 + 1) as u8;
            let x = state.next_u64() as usize % (self.grid_size as usize - w as usize + 1);
            let y = state.next_u64() as usize % (self.grid_size as usize - h as usize + 1);
            for dx in 0..w {
                for dy in 0..h {
                    let idx = (y + dy as usize) * self.grid_size as usize + (x + dx as usize);
                    if idx < pixels.len() {
                        pixels[idx] = color;
                    }
                }
            }
        }
        ArcGrid::new(self.grid_size, self.grid_size, pixels)
    }

    fn random_program(&self, state: &mut SplitMix64, node_id: &str) -> Program {
        let num_steps = state.next_u64() as usize % self.max_operators + 1;
        let mut steps = Vec::new();
        for _ in 0..num_steps {
            let op = state.next_u64() % 6;
            match op {
                0 => steps.push(Transformation::Recolor {
                    node_id: node_id.to_string(),
                    new_color: ((state.next_u64() % 9 + 1) as u8).to_string(),
                }),
                1 => steps.push(Transformation::Translate {
                    node_id: node_id.to_string(),
                    dx: (state.next_u64() as i64 % 3) - 1,
                    dy: (state.next_u64() as i64 % 3) - 1,
                }),
                2 => steps.push(Transformation::Rotate {
                    node_id: node_id.to_string(),
                    angle: 90,
                }),
                3 => steps.push(Transformation::Delete {
                    node_id: node_id.to_string(),
                }),
                4 => steps.push(Transformation::Create {
                    color: ((state.next_u64() % 9 + 1) as u8).to_string(),
                    bbox_x: state.next_u64() % self.grid_size as u64,
                    bbox_y: state.next_u64() % self.grid_size as u64,
                    bbox_w: (state.next_u64() % 2 + 1) as u8,
                    bbox_h: (state.next_u64() % 2 + 1) as u8,
                }),
                _ => {} // NoOp vagy bármilyen más érték
            }
        }
        Program::new(steps)
    }

    pub fn generate_task(&self) -> (ArcGrid, ArcGrid, Program, KernelStructureGraph) {
        let mut state = SplitMix64::new(self.seed);
        let input_grid = self.random_grid(&mut state);
        let input_ksg = ArcAdapter::grid_to_ksg(&input_grid);
        let node_id = if let Some(node) = input_ksg.nodes.first() {
            node.id.clone()
        } else {
            "obj_0".to_string()
        };
        let program = self.random_program(&mut state, &node_id);
        let output_graph = program.apply(&input_ksg);
        let output_grid = ArcAdapter::ksg_to_grid(&output_graph, self.grid_size, self.grid_size, 0);
        (input_grid, output_grid, program, input_ksg)
    }
}
