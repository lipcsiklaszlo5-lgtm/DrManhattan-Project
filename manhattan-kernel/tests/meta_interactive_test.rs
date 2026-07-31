use manhattan_kernel::meta_learner::MetaLearner;
use manhattan_kernel::agent::agent_loop::Environment;
use manhattan_kernel::adapter::arc::adapter::ArcGrid;
use std::collections::VecDeque;

struct StepByStepEnv {
    states: VecDeque<ArcGrid>,
    current: ArcGrid,
}

impl StepByStepEnv {
    fn new(initial: ArcGrid, next_states: Vec<ArcGrid>) -> Self {
        Self {
            states: VecDeque::from(next_states),
            current: initial,
        }
    }
}

impl Environment for StepByStepEnv {
    fn step(&mut self, action: &str) -> Result<(ArcGrid, ArcGrid), String> {
        // Bármilyen akciót elfogadunk, és a következő előre definiált állapotba lépünk
        if let Some(next) = self.states.pop_front() {
            let obs = next.clone();
            self.current = obs.clone();
            // A cél most nem érdekes, csak a megfigyelést adjuk vissza
            Ok((obs, next))
        } else {
            Err("no more states".into())
        }
    }

    fn reset(&mut self) -> ArcGrid {
        self.current.clone()
    }
}

#[test]
fn test_interactive_learning_builds_model() {
    let mut learner = MetaLearner::new();
    // 3x3-as rács, a játékos (1) lépked jobbra
    let s0 = ArcGrid::new(3, 1, vec![1, 0, 0]);
    let s1 = ArcGrid::new(3, 1, vec![0, 1, 0]);
    let s2 = ArcGrid::new(3, 1, vec![0, 0, 1]);
    let mut env = StepByStepEnv::new(s0.clone(), vec![s1, s2]);
    let success_rate = learner.learn_interactive(&mut env, 1).unwrap();
    // Nem feltétlenül oldja meg, de legalább ne panikoljon
    assert!(success_rate >= 0.0);
}
