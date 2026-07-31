use manhattan_kernel::policy::{PolicyEngine, CostModel};
use manhattan_kernel::candidate::CandidateGenerator;
use manhattan_kernel::telemetry::Telemetry;
use manhattan_kernel::agent::agent_loop::Environment;
use manhattan_kernel::adapter::arc::adapter::ArcGrid;

struct SimpleGame {
    player_pos: (usize, usize),
    exit_pos: (usize, usize),
    width: usize,
    height: usize,
}

impl SimpleGame {
    fn new() -> Self {
        Self { player_pos: (0, 0), exit_pos: (2, 0), width: 3, height: 1 }
    }
}

impl Environment for SimpleGame {
    fn step(&mut self, action: &str) -> Result<(ArcGrid, ArcGrid), String> {
        let parts: Vec<&str> = action.split('_').collect();
        if parts.len() >= 4 && parts[0] == "translate" {
            let dy: i64 = parts.last().unwrap().parse().unwrap_or(0);
            let dx: i64 = parts.get(parts.len()-2).unwrap().parse().unwrap_or(0);
            let new_x = (self.player_pos.0 as i64 + dx) as usize;
            let new_y = (self.player_pos.1 as i64 + dy) as usize;
            if new_x < self.width && new_y < self.height {
                self.player_pos = (new_x, new_y);
            }
        }
        let mut pixels = vec![0u8; self.width * self.height];
        pixels[self.player_pos.1 * self.width + self.player_pos.0] = 1;
        pixels[self.exit_pos.1 * self.width + self.exit_pos.0] = 2;
        let obs = ArcGrid::new(self.width as u8, self.height as u8, pixels.clone());
        let target = if self.player_pos == self.exit_pos {
            let mut tpixels = vec![0u8; self.width * self.height];
            tpixels[self.exit_pos.1 * self.width + self.exit_pos.0] = 3;
            ArcGrid::new(self.width as u8, self.height as u8, tpixels)
        } else {
            obs.clone()
        };
        Ok((obs, target))
    }

    fn reset(&mut self) -> ArcGrid {
        self.player_pos = (0, 0);
        let mut pixels = vec![0u8; self.width * self.height];
        pixels[self.player_pos.1 * self.width + self.player_pos.0] = 1;
        pixels[self.exit_pos.1 * self.width + self.exit_pos.0] = 2;
        ArcGrid::new(self.width as u8, self.height as u8, pixels)
    }
}

#[test]
fn test_policy_interactive_task() {
    let cost_model = CostModel { llm_cost_per_call: 0.1 };
    let candidate_gen = CandidateGenerator::new(3);
    let mut engine = PolicyEngine::new(cost_model, candidate_gen);
    let mut telemetry = Telemetry::new();
    let mut game = SimpleGame::new();
    let result = engine.execute_interactive_task(&mut game, &mut telemetry, 10);
    assert!(result.is_ok(), "Interactive task should be solved");
}
