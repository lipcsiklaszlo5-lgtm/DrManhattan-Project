use manhattan_kernel::agent::AgentLoop;
use manhattan_kernel::agent::agent_loop::Environment;
use manhattan_kernel::adapter::arc::adapter::ArcGrid;

struct MockEnv {
    pub step_count: usize,
}

impl Environment for MockEnv {
    fn step(&mut self, action: &str) -> Result<(ArcGrid, ArcGrid), String> {
        if action == "solve" && self.step_count == 0 {
            self.step_count += 1;
            let mut target = ArcGrid::new(2, 2, vec![0, 0, 0, 1]);
            let obs = ArcGrid::new(2, 2, vec![0, 0, 0, 0]);
            Ok((obs, target))
        } else {
            Err("unknown action".into())
        }
    }

    fn reset(&mut self) -> ArcGrid {
        self.step_count = 0;
        ArcGrid::new(2, 2, vec![0, 0, 0, 0])
    }
}

#[test]
fn test_agent_loop_learns_program() {
    let mut agent = AgentLoop::new();
    let mut env = MockEnv { step_count: 0 };
    let result = agent.run_episode(&mut env, 5);
    assert!(result.is_ok(), "Agent should learn a program");
    let program = result.unwrap();
    assert!(!program.steps.is_empty(), "Program should have at least one step");
    assert!(agent.learned_program.is_some());
}
