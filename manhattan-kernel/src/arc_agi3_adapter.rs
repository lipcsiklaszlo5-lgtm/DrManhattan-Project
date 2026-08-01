use crate::adapter::arc::adapter::{ArcGrid, ArcAdapter};
use crate::meta_learner::{MetaLearner, TaskInstance};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Egy ARC-AGI-3 feladat JSON reprezentációja
#[derive(Debug, Deserialize)]
pub struct ArcAgi3Task {
    pub task_id: String,
    pub description: String,
    pub input: Vec<Vec<u8>>,
    pub output: Vec<Vec<u8>>,
}

/// Adapter, ami beolvassa az ARC-AGI-3 JSON fájlt és TaskInstance-okká alakítja
pub struct ArcAgi3Adapter;

impl ArcAgi3Adapter {
    /// Beolvas egy JSON fájlt és visszaadja a feladatok listáját
    pub fn load_tasks(path: &Path) -> Result<Vec<ArcAgi3Task>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let tasks: Vec<ArcAgi3Task> = serde_json::from_str(&content)?;
        Ok(tasks)
    }

    /// Átalakít egy ArcAgi3Task-ot TaskInstance-á
    pub fn to_task_instance(task: &ArcAgi3Task) -> TaskInstance {
        let input_grid = Self::grid_from_2d(&task.input);
        let output_grid = Self::grid_from_2d(&task.output);
        TaskInstance {
            grid: input_grid,
            target: output_grid,
        }
    }

    /// Segédfüggvény: 2D vektor → ArcGrid
    fn grid_from_2d(data: &[Vec<u8>]) -> ArcGrid {
        let height = data.len() as u8;
        let width = if height > 0 { data[0].len() as u8 } else { 0 };
        let pixels: Vec<u8> = data.iter().flatten().cloned().collect();
        ArcGrid::new(width, height, pixels)
    }
}
