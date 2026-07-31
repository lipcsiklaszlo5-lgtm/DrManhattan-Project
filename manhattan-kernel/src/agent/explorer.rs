use crate::structure::KernelStructureGraph;
use crate::sandbox::operators::{Transformation, apply_transformation};
use crate::abstraction::program::ProgramSynthesizer;
use crate::adapter::arc::adapter::ArcGrid;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;

pub struct ExplorerAgent {
    pub synthesizer: ProgramSynthesizer,
    pub world_model: HashMap<(u64, String), KernelStructureGraph>,
    pub action_history: Vec<(KernelStructureGraph, String, KernelStructureGraph)>,
    rng: rand::rngs::ThreadRng,
}

impl ExplorerAgent {
    pub fn new() -> Self {
        Self {
            synthesizer: ProgramSynthesizer::new(),
            world_model: HashMap::new(),
            action_history: Vec::new(),
            rng: thread_rng(),
        }
    }

    pub fn possible_actions(&self, graph: &KernelStructureGraph) -> Vec<String> {
        let mut actions = Vec::new();
        for node in &graph.nodes {
            if node.node_type == "arc_object" {
                actions.push(format!("translate_{}_0_-1", node.id));
                actions.push(format!("translate_{}_0_1", node.id));
                actions.push(format!("translate_{}_-1_0", node.id));
                actions.push(format!("translate_{}_1_0", node.id));
            }
        }
        actions
    }

    pub fn parse_action(&self, action: &str, _graph: &KernelStructureGraph) -> Option<Transformation> {
        let parts: Vec<&str> = action.split('_').collect();
        if parts.len() >= 4 && parts[0] == "translate" {
            let dy: i64 = parts.last()?.parse().ok()?;
            let dx: i64 = parts.get(parts.len()-2)?.parse().ok()?;
            let node_id = parts[1..parts.len()-2].join("_");
            Some(Transformation::Translate { node_id, dx, dy })
        } else {
            None
        }
    }

    pub fn try_action(&self, graph: &KernelStructureGraph, action: &str) -> Result<KernelStructureGraph, String> {
        if let Some(transform) = self.parse_action(action, graph) {
            let new_graph = apply_transformation(graph, &transform);
            for node in &new_graph.nodes {
                if let (Some(x), Some(y)) = (node.attributes.get("bbox_x"), node.attributes.get("bbox_y")) {
                    let x: i64 = x.parse().unwrap_or(0);
                    let y: i64 = y.parse().unwrap_or(0);
                    if x < 0 || y < 0 { return Err("out of bounds".into()); }
                }
            }
            Ok(new_graph)
        } else {
            Err("unknown action".into())
        }
    }

    pub fn explore_to_target(
        &mut self,
        start_grid: &ArcGrid,
        target_grid: &ArcGrid,
        max_steps: usize,
    ) -> Result<Vec<String>, String> {
        let start_ksg = crate::adapter::arc::adapter::ArcAdapter::grid_to_ksg(start_grid);
        let target_ksg = crate::adapter::arc::adapter::ArcAdapter::grid_to_ksg(target_grid);
        let mut current = start_ksg.clone();
        let mut plan = Vec::new();

        for _ in 0..max_steps {
            let fp = current.fingerprint();
            if let Some(program) = self.synthesizer.find_best_program(&current, &target_ksg) {
                let mut temp = current.clone();
                for step in &program.steps {
                    temp = apply_transformation(&temp, step);
                    plan.push(format!("{:?}", step));
                }
                if temp.nodes == target_ksg.nodes {
                    return Ok(plan);
                }
            }

            let actions = self.possible_actions(&current);
            if actions.is_empty() { break; }
            let action = actions.choose(&mut self.rng).unwrap().clone();
            if let Ok(new_ksg) = self.try_action(&current, &action) {
                self.world_model.insert((fp, action.clone()), new_ksg.clone());
                self.action_history.push((current.clone(), action.clone(), new_ksg.clone()));
                plan.push(action.clone());
                current = new_ksg;
                if current.nodes == target_ksg.nodes {
                    self.synthesizer.learn_from_example(&start_ksg, &current);
                    return Ok(plan);
                }
            }
        }
        Err("failed to reach target".into())
    }
}
