use crate::hypothesis_bus::HypothesisBus;
use crate::abstraction::program::{Program, ProgramSynthesizer};
use crate::adapter::arc::adapter::ArcGrid;

pub trait Environment {
    fn step(&mut self, action: &str) -> Result<(ArcGrid, ArcGrid), String>;
    fn reset(&mut self) -> ArcGrid;
}

pub struct AgentLoop {
    pub hypothesis_bus: HypothesisBus,
    pub synthesizer: ProgramSynthesizer,
    pub learned_program: Option<Program>,
}

impl AgentLoop {
    pub fn new() -> Self {
        Self {
            hypothesis_bus: HypothesisBus::new(),
            synthesizer: ProgramSynthesizer::new(),
            learned_program: None,
        }
    }

    pub fn run_episode(&mut self, env: &mut dyn Environment, max_steps: usize) -> Result<Program, String> {
        let obs = env.reset();
        for _step in 0..max_steps {
            if let Ok((_new_obs, target_grid)) = env.step("solve") {
                let obs_ksg = crate::adapter::arc::adapter::ArcAdapter::grid_to_ksg(&obs);
                let target_ksg = crate::adapter::arc::adapter::ArcAdapter::grid_to_ksg(&target_grid);
                if let Some(program) = self.synthesizer.learn_from_example(&obs_ksg, &target_ksg) {
                    self.learned_program = Some(program.clone());
                    return Ok(program);
                }
            }
        }
        Err("Failed to learn program within max steps".into())
    }
}
