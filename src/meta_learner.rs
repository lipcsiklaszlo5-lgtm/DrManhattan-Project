use crate::agent::agent_loop::{AgentLoop, Environment};
use crate::agent::explorer::ExplorerAgent;
use crate::abstraction::program::ProgramSynthesizer;
use crate::abstraction::hypothesis::HypothesisManager;
use crate::abstraction::goal_decomposer::GoalDecomposer;
use crate::concept::{ConceptRegistry, Concept};
use crate::concept_learner::ConceptLearner;
use crate::adapter::arc::adapter::{ArcGrid, ArcAdapter};
use std::collections::HashMap;

pub struct TaskInstance {
    pub grid: ArcGrid,
    pub target: ArcGrid,
}

pub struct MetaLearner {
    pub agent: AgentLoop,
    pub explorer: ExplorerAgent,
    pub hypothesis_manager: HypothesisManager,
    pub program_synthesizer: ProgramSynthesizer,
    pub concept_registry: ConceptRegistry,
    pub concept_learner: ConceptLearner,
    pub task_stats: HashMap<String, (u32, u32)>,
}

impl MetaLearner {
    pub fn new() -> Self {
        Self {
            agent: AgentLoop::new(),
            explorer: ExplorerAgent::new(),
            hypothesis_manager: HypothesisManager::new(),
            program_synthesizer: ProgramSynthesizer::new(),
            concept_registry: ConceptRegistry::default(),
            concept_learner: ConceptLearner::new(),
            task_stats: HashMap::new(),
        }
    }

    pub fn learn_from_task(&mut self, task: TaskInstance) -> bool {
        struct OneShotEnv {
            obs: ArcGrid,
            target: ArcGrid,
            solved: bool,
        }
        impl Environment for OneShotEnv {
            fn step(&mut self, action: &str) -> Result<(ArcGrid, ArcGrid), String> {
                if action == "solve" && !self.solved {
                    self.solved = true;
                    Ok((self.obs.clone(), self.target.clone()))
                } else {
                    Err("already solved or unknown action".into())
                }
            }
            fn reset(&mut self) -> ArcGrid { self.obs.clone() }
        }

        let mut env = OneShotEnv { obs: task.grid.clone(), target: task.target.clone(), solved: false };

        // Próbáljuk megoldani az egy-lövéses tanulással vagy explorációval
        let solved = match self.agent.run_episode(&mut env, 5) {
            Ok(_program) => true,
            Err(_) => {
                match self.explorer.explore_to_target(&task.grid, &task.target, 20) {
                    Ok(_plan) => {
                        // Az explorer megtalálta az útvonalat, de itt is tanulunk a diffből
                        let input_ksg = ArcAdapter::grid_to_ksg(&task.grid);
                        let target_ksg = ArcAdapter::grid_to_ksg(&task.target);
                        self.program_synthesizer.learn_from_example(&input_ksg, &target_ksg);
                        true
                    }
                    Err(_) => false,
                }
            }
        };

        if solved {
            // Pixel-szintű validáció a legjobb tanult programmal
            let input_ksg = ArcAdapter::grid_to_ksg(&task.grid);
            let target_ksg = ArcAdapter::grid_to_ksg(&task.target);
            let best_program = self.program_synthesizer.find_best_program(&input_ksg, &target_ksg);

            if let Some(prog) = best_program {
                let result_ksg = prog.apply(&input_ksg);
                let result_grid = ArcAdapter::ksg_to_grid(&result_ksg, task.target.width, task.target.height, 0);
                if result_grid.pixels == task.target.pixels {
                    // Valódi siker
                    self.hypothesis_manager.process_grid(&task.grid, &mut self.program_synthesizer, Some(&target_ksg));
                    let rep_name = self.hypothesis_manager.best_hypothesis().map(|h| h.representation_name.clone());
                    if let Some(name) = rep_name {
                        self.hypothesis_manager.record_success(&name);
                    }
                    self.analyze_and_adapt(&task.grid, &task.target);
                    let key = "agent_one_shot".to_string();
                    let entry = self.task_stats.entry(key).or_insert((0,0));
                    entry.0 += 1; entry.1 += 1;
                    return true;
                }
            }
        }

        // Ha nem sikerült, adaptálódunk, de sikertelennek jelöljük
        self.analyze_and_adapt(&task.grid, &task.target);
        let key = "agent_one_shot".to_string();
        let entry = self.task_stats.entry(key).or_insert((0,0));
        entry.1 += 1;
        false
    }

    pub fn learn_interactive(&mut self, env: &mut dyn Environment, max_episodes: usize) -> Result<f64, String> {
        let mut total_success = 0u32;
        let mut total_attempts = 0u32;

        for _ep in 0..max_episodes {
            total_attempts += 1;
            let obs = env.reset();
            let mut current_ksg = ArcAdapter::grid_to_ksg(&obs);
            for _step in 0..10 {
                let actions = self.explorer.possible_actions(&current_ksg);
                if actions.is_empty() { break; }
                use rand::seq::SliceRandom;
                let action = actions.choose(&mut rand::thread_rng()).unwrap().clone();
                if let Ok((new_obs, _target)) = env.step(&action) {
                    let new_ksg = ArcAdapter::grid_to_ksg(&new_obs);
                    self.program_synthesizer.learn_from_example(&current_ksg, &new_ksg);
                    current_ksg = new_ksg;
                }
            }
            if self.program_synthesizer.programs.len() > total_success as usize {
                total_success += 1;
            }
        }
        Ok(total_success as f64 / total_attempts as f64)
    }

    fn analyze_and_adapt(&mut self, input: &ArcGrid, target: &ArcGrid) {
        let input_ksg = ArcAdapter::grid_to_ksg(input);
        let target_ksg = ArcAdapter::grid_to_ksg(target);
        let subgoals = GoalDecomposer::decompose(&input_ksg, &target_ksg);
        if !subgoals.is_empty() {
            for sg in &subgoals {
                self.program_synthesizer.learn_from_example(&input_ksg, &sg.target_ksg);
            }
        }
        let new_concepts = self.concept_learner.learn_from_diff(&input_ksg, &target_ksg, &mut self.concept_registry);
        for concept in new_concepts {
            println!("Discovered new concept: {:?}", concept);
            self.concept_registry.add_concept(concept.clone());
            self.program_synthesizer.map_concept_to_transform(concept, "Translate");
        }
        let static_concepts = self.discover_concepts(&input_ksg, &target_ksg);
        for concept in static_concepts {
            println!("Static concept detected: {:?}", concept);
        }
    }

    fn discover_concepts(&self, _input_ksg: &crate::structure::KernelStructureGraph, target_ksg: &crate::structure::KernelStructureGraph) -> Vec<Concept> {
        let mut new_concepts = Vec::new();
        for edge in &target_ksg.edges {
            if edge.rel_type == "contains" && !self.concept_registry.scan(target_ksg).contains(&Concept::Hole) {
                new_concepts.push(Concept::Hole);
            }
        }
        if target_ksg.nodes.iter().any(|n| n.attributes.contains_key("symmetry")) &&
           !self.concept_registry.scan(target_ksg).contains(&Concept::Symmetry) {
            new_concepts.push(Concept::Symmetry);
        }
        new_concepts
    }
}
