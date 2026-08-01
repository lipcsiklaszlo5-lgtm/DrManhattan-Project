use crate::adapter::arc::adapter::ArcGrid;
use crate::sandbox::operators::Transformation;
use rand::Rng;

pub struct SyntheticArcGenerator {
    rng: rand::rngs::ThreadRng,
}

impl SyntheticArcGenerator {
    pub fn new() -> Self {
        Self { rng: rand::thread_rng() }
    }

    /// Generál egy véletlenszerű ARC feladatot (input, target) és a hozzá tartozó transzformációkat.
    pub fn generate_task(&mut self, width: u8, height: u8, num_objects: usize, num_operations: usize) -> (ArcGrid, ArcGrid, Vec<Transformation>) {
        let mut pixels = vec![0u8; (width as usize) * (height as usize)];
        let mut rng = &mut self.rng;
        // Hozzunk létre véletlenszerű objektumokat
        for i in 0..num_objects {
            let x = rng.gen_range(0..width) as usize;
            let y = rng.gen_range(0..height) as usize;
            let color = rng.gen_range(1..=9) as u8;
            pixels[y * width as usize + x] = color;
        }

        let input = ArcGrid::new(width, height, pixels);
        let mut target = input.clone();
        let mut operations = Vec::new();

        // Alkalmazzunk véletlenszerű transzformációkat
        for _ in 0..num_operations {
            if target.pixels.iter().all(|&p| p == 0) { break; }
            let op = self.random_operation(&target);
            if let Some(new_grid) = crate::sandbox::operators::apply_transformation_to_grid(&target, &op) {
                target = new_grid;
                operations.push(op);
            }
        }

        (input, target, operations)
    }

    fn random_operation(&mut self, grid: &ArcGrid) -> Transformation {
        let mut rng = &mut self.rng;
        let objects: Vec<_> = grid.pixels.iter()
            .enumerate()
            .filter(|(_, &c)| c != 0)
            .map(|(i, &c)| (i, c))
            .collect();
        if objects.is_empty() {
            return Transformation::NoOp;
        }
        let idx = rng.gen_range(0..objects.len());
        let (pos, color) = objects[idx];
        let x = (pos % grid.width as usize) as u8;
        let y = (pos / grid.width as usize) as u8;
        let node_id = format!("obj_{}", pos);
        match rng.gen_range(0..5) {
            0 => Transformation::Translate {
                node_id,
                dx: rng.gen_range(-2..=2) as i64,
                dy: rng.gen_range(-2..=2) as i64,
            },
            1 => Transformation::Recolor {
                node_id,
                new_color: rng.gen_range(1..=9).to_string(),
            },
            2 => Transformation::Delete { node_id },
            3 => Transformation::Create {
                color: rng.gen_range(1..=9).to_string(),
                bbox_x: (x as i64 + rng.gen_range(-1i64..=1)) as u64,
                bbox_y: (y as i64 + rng.gen_range(-1i64..=1)) as u64,
                bbox_w: 1,
                bbox_h: 1,
            },
            4 => Transformation::Rotate {
                node_id,
                angle: match rng.gen_range(0..3) {
                    0 => 90,
                    1 => 180,
                    _ => 270,
                },
            },
            _ => Transformation::NoOp,
        }
    }
}
