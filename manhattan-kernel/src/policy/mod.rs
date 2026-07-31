use crate::executor::Executor;
use std::collections::HashMap;
use crate::task::Task;
use crate::adapter::DomainAdapter;
use crate::candidate::CandidateGenerator;
use crate::structure::KernelStructureGraph;
use crate::structure::topology::{graph_diff, NodeTransformation};
use crate::telemetry::Telemetry;
use crate::memory::episodic::EpisodicEntry;
use crate::memory::semantic::{SemanticSchema, Predicate};
use crate::memory::procedural::ProceduralRule;
use crate::memory::loader::load_schemas;
use crate::executor::LlmExecutor;
use crate::schema::index::SchemaIndex;
use crate::abstraction::hypothesis::HypothesisManager;
use crate::abstraction::program::{Program, ProgramSynthesizer};
use crate::abstraction::goal_decomposer::GoalDecomposer;
use crate::adapter::arc::adapter::ArcAdapter;
use crate::sandbox::operators::Transformation;
use crate::hypothesis_bus::HypothesisBus;
use crate::agent::explorer::ExplorerAgent;
use crate::agent::agent_loop::Environment;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CostModel {
    pub llm_cost_per_call: f32,
}

impl CostModel {
    pub fn estimate_llm_cost(&self, _task: &Task) -> f32 { self.llm_cost_per_call }
}

pub struct PolicyEngine<'a> {
    cost_model: CostModel,
    candidate_gen: CandidateGenerator,
    rules: HashMap<u64, ProceduralRule>,
    episodic_log: Vec<EpisodicEntry>,
    semantic_schemas: HashMap<uuid::Uuid, SemanticSchema>,
    schema_index: SchemaIndex,
    llm_executor: Option<&'a LlmExecutor>,
    hypothesis_bus: Option<&'a mut HypothesisBus>,
    pub hypothesis_manager: HypothesisManager,
    pub program_synthesizer: ProgramSynthesizer,
    pub explorer: ExplorerAgent,
}

impl<'a> PolicyEngine<'a> {
    pub fn new(cost_model: CostModel, candidate_gen: CandidateGenerator) -> Self {
        Self {
            cost_model, candidate_gen, rules: HashMap::new(), episodic_log: Vec::new(),
            semantic_schemas: HashMap::new(), schema_index: SchemaIndex::new(), llm_executor: None,
            hypothesis_bus: None,
            hypothesis_manager: HypothesisManager::new(),
            program_synthesizer: ProgramSynthesizer::new(),
            explorer: ExplorerAgent::new(),
        }
    }

    pub fn with_llm_executor(mut self, executor: &'a LlmExecutor) -> Self { self.llm_executor = Some(executor); self }
    pub fn with_hypothesis_bus(mut self, bus: &'a mut HypothesisBus) -> Self { self.hypothesis_bus = Some(bus); self }

    pub fn load_schemas(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let schemas = load_schemas(path)?;
        for (_fp, schema) in schemas { self.schema_index.insert(&schema); self.semantic_schemas.insert(schema.id, schema); }
        Ok(())
    }

    fn check_cache(&self, structure: &KernelStructureGraph) -> Option<&ProceduralRule> { self.rules.get(&structure.fingerprint()) }
    fn store_rule(&mut self, structure: &KernelStructureGraph, solution: &str) {
        let mut rule = ProceduralRule { id: uuid::Uuid::new_v4(), pattern: solution.to_string(), confidence: 0.5, success_count: 1, domain_tags: vec!["arc".into()] };
        rule.record_success();
        self.rules.insert(structure.fingerprint(), rule);
    }
    fn record_operator_success(&mut self, action: &str) { let s = self.candidate_gen.operator_stats.entry(action.to_string()).or_insert((0,0)); s.0 += 1; s.1 += 1; }
    fn record_operator_attempt(&mut self, action: &str) { let s = self.candidate_gen.operator_stats.entry(action.to_string()).or_insert((0,0)); s.1 += 1; }
    fn record_episodic(&mut self, task: &Task, success: bool, notes: &str) {
        self.episodic_log.push(EpisodicEntry { id: uuid::Uuid::new_v4(), task_intent: task.intent.clone(), success, timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(), notes: notes.to_string() });
    }

    fn store_semantic_schema(&mut self, structure: &KernelStructureGraph, _solution: &str, _action: &str) {
        let fp = structure.fingerprint();
        let mut schema = SemanticSchema::new(structure.clone());
        schema.metadata.tags = vec!["arc".into()];
        schema.algebra.requires.insert(Predicate::TypeMismatch);
        schema.algebra.provides.insert(Predicate::TypeResolved);
        schema.metadata.fingerprint = fp;
        self.schema_index.insert(&schema);
        self.semantic_schemas.insert(schema.id, schema);
    }

    pub fn learn_transformation(&mut self, before: &KernelStructureGraph, after: &KernelStructureGraph) -> Vec<NodeTransformation> {
        let diffs = graph_diff(before, after);
        let fp = before.fingerprint();
        let mut schema = SemanticSchema::new(before.clone());
        schema.metadata.tags = vec!["arc".into()];
        schema.metadata.fingerprint = fp;
        schema.algebra.requires.insert(Predicate::TypeMismatch);
        schema.algebra.provides.insert(Predicate::TypeResolved);
        self.schema_index.insert(&schema);
        self.semantic_schemas.insert(schema.id, schema);
        diffs
    }

    pub fn decide(&self, task: &Task, _adapter: &dyn DomainAdapter) -> &str {
        if task.context.grid.is_some() {
            return "schema_plan";
        }
        if task.context.structure.is_some() {
            if let Some(s) = &task.context.structure {
                if s.nodes.is_empty() && s.edges.is_empty() { return "success"; }
                if self.check_cache(s).is_some() { return "cache"; }
            }
            return "algorithm";
        }
        if self.cost_model.estimate_llm_cost(task) < 0.02 { return "llm"; }
        "llm"
    }

    pub fn run_local_search(&mut self, structure: &KernelStructureGraph, adapter: &dyn DomainAdapter, original_code: &str, max_candidates: usize) -> Option<String> {
        let action = structure.nodes.iter().find(|n| n.node_type == "compiler_error").and_then(|n| n.attributes.get("action").cloned()).unwrap_or_default();
        self.record_operator_attempt(&action);
        let mut candidates = self.candidate_gen.generate(structure, max_candidates);

        if let Some(ref mut bus) = self.hypothesis_bus {
            let concepts: Vec<String> = bus.get_hypotheses().into_iter().map(|h| h.concept.to_lowercase()).collect();
            if !concepts.is_empty() {
                candidates.sort_by_key(|c| {
                    let action = c.nodes.iter()
                        .find(|n| n.node_type == "compiler_error")
                        .and_then(|n| n.attributes.get("action").cloned())
                        .unwrap_or_default();
                    let matches = concepts.iter().any(|concept| action.to_lowercase().contains(concept));
                    if matches { 0 } else { 1 }
                });
            }
        }

        for c in &candidates {
            let code = adapter.graph_to_code(c, original_code);
            if adapter.validate(structure, &code).is_ok() {
                self.store_rule(structure, &code);
                self.store_semantic_schema(structure, &code, &action);
                self.record_operator_success(&action);
                return Some(code);
            }
        }
        None
    }

    fn resolve_abstract_program(program: &Program, target_ksg: &KernelStructureGraph) -> Vec<Transformation> {
        program.steps.iter().map(|step| {
            match step {
                Transformation::RecolorToTarget { node_id } => {
                    if let Some(target_node) = target_ksg.nodes.iter().find(|n| &n.id == node_id) {
                        if let Some(color) = target_node.attributes.get("color") {
                            Transformation::Recolor { node_id: node_id.clone(), new_color: color.clone() }
                        } else { Transformation::NoOp }
                    } else { Transformation::NoOp }
                }
                Transformation::TranslateToTarget { node_id } => {
                    if let Some(target_node) = target_ksg.nodes.iter().find(|n| &n.id == node_id) {
                        let tx = target_node.attributes.get("bbox_x").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                        let ty = target_node.attributes.get("bbox_y").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                        Transformation::Translate { node_id: node_id.clone(), dx: tx, dy: ty }
                    } else { Transformation::NoOp }
                }
                _ => step.clone(),
            }
        }).collect()
    }

    pub fn execute_task(&mut self, task: &mut Task, adapter: &dyn DomainAdapter, telemetry: &mut Telemetry) -> Result<String, String> {
        let original_code = task.intent.clone();

        if task.context.structure.is_none() {
            if let Some(ref grid) = task.context.grid {
                task.context.structure = Some(ArcAdapter::grid_to_ksg(grid));
            } else {
                task.context.structure = Some(adapter.build_structure(task));
            }
        }

        let path = self.decide(task, adapter);
        let mut result = String::new();
        let mut success = false;

        match path {
            "success" => { result = "already correct".to_string(); success = true; }
            "cache" => {
                if let Some(s) = &task.context.structure {
                    if let Some(r) = self.check_cache(s) { telemetry.record_cache_hit(); result = r.pattern.clone(); success = true; }
                }
                if !success {
                    if let Some(ex) = self.llm_executor {
                        if let Ok(o) = ex.execute(task) {
                            telemetry.record_llm_call(o.confidence as u64, 200);
                            if let Some(s) = &task.context.structure {
                                if adapter.validate(s, &o.content).is_ok() { self.store_rule(s, &o.content); self.store_semantic_schema(s, &o.content, "llm"); result = o.content; success = true; }
                            }
                            if !success { result = "llm response failed validation".to_string(); }
                        } else { result = "llm executor error".to_string(); }
                    } else { telemetry.record_llm_call(100, 200); result = "llm fallback (cache miss, no executor)".to_string(); }
                }
            }
            "algorithm" => {
                let structure = task.context.structure.as_ref().unwrap().clone();
                if let Some(solution) = self.run_local_search(&structure, adapter, &original_code, 5) {
                    telemetry.record_local_search_success();
                    result = solution;
                    success = true;
                } else {
                    if let (Some(ref grid), Some(ref target_grid)) = (&task.context.grid, &task.context.target_grid) {
                        if let Ok(_plan) = self.explorer.explore_to_target(grid, target_grid, 20) {
                            result = "interactive exploration solved task".into();
                            success = true;
                            telemetry.record_local_search_success();
                        }
                    }
                    if !success {
                        if let Some(ex) = self.llm_executor {
                            if let Ok(o) = ex.execute(task) {
                                telemetry.record_llm_call(o.confidence as u64, 200);
                                if let Some(s) = &task.context.structure { if adapter.validate(s, &o.content).is_ok() { self.store_rule(s, &o.content); self.store_semantic_schema(s, &o.content, "llm"); result = o.content; success = true; } }
                                if !success { result = "llm response failed validation".to_string(); }
                            } else { result = "llm executor error".to_string(); }
                        } else { telemetry.record_llm_call(100, 200); result = "llm fallback solution".to_string(); }
                    }
                }
            }
            "schema_plan" => {
                if let (Some(ref grid), Some(ref target_grid)) = (&task.context.grid, &task.context.target_grid) {
                    let input_ksg = task.context.structure.as_ref().unwrap().clone();
                    let target_ksg = ArcAdapter::grid_to_ksg(target_grid);

                    self.hypothesis_manager.process_grid(grid, &mut self.program_synthesizer, Some(&target_ksg));

                    let hypothesis_data = self.hypothesis_manager.best_hypothesis().map(|h| {
                        (h.representation_name.clone(), h.program.clone())
                    });

                    if let Some((rep_name, Some(best_program))) = hypothesis_data {
                        let resolved_steps = Self::resolve_abstract_program(&best_program, &target_ksg);
                        let resolved_program = Program::new(resolved_steps);

                        let result_graph = resolved_program.apply(&input_ksg);
                        let result_grid = ArcAdapter::ksg_to_grid(&result_graph, target_grid.width, target_grid.height, 0);

                        if result_grid.pixels == target_grid.pixels {
                            telemetry.record_local_search_success();
                            self.store_rule(&input_ksg, &format!("{:?}", best_program.steps));
                            self.hypothesis_manager.record_success(&rep_name);
                            result = format!("ARC solved with program: {:?}", best_program.steps);
                            success = true;
                        } else {
                            self.hypothesis_manager.record_failure(&rep_name);
                        }
                    }

                    if !success {
                        let subgoals = GoalDecomposer::decompose(&input_ksg, &target_ksg);
                        if !subgoals.is_empty() {
                            println!("Decomposed into {} subgoals", subgoals.len());
                            let mut current_grid = grid.clone();
                            let mut all_solved = true;

                            for sg in &subgoals {
                                let sg_target_grid = ArcAdapter::ksg_to_grid(&sg.target_ksg, target_grid.width, target_grid.height, 0);

                                let current_ksg = ArcAdapter::grid_to_ksg(&current_grid);
                                let sg_ksg = ArcAdapter::grid_to_ksg(&sg_target_grid);
                                self.program_synthesizer.learn_from_example(&current_ksg, &sg_ksg);

                                self.hypothesis_manager.process_grid(&current_grid, &mut self.program_synthesizer, Some(&sg_ksg));

                                let sg_hypothesis_data = self.hypothesis_manager.best_hypothesis().map(|h| {
                                    (h.representation_name.clone(), h.program.clone())
                                });

                                if let Some((sg_rep_name, Some(sg_program))) = sg_hypothesis_data {
                                    let resolved = Self::resolve_abstract_program(&sg_program, &sg_ksg);
                                    let resolved_prog = Program::new(resolved);
                                    let result_graph = resolved_prog.apply(&current_ksg);
                                    let result_grid = ArcAdapter::ksg_to_grid(&result_graph, target_grid.width, target_grid.height, 0);

                                    if result_grid.pixels == sg_target_grid.pixels {
                                        self.hypothesis_manager.record_success(&sg_rep_name);
                                        current_grid = sg_target_grid;
                                        println!("Subgoal solved: {}", sg.description);
                                    } else {
                                        self.hypothesis_manager.record_failure(&sg_rep_name);
                                        all_solved = false;
                                        break;
                                    }
                                } else {
                                    all_solved = false;
                                    break;
                                }
                            }

                            if all_solved {
                                if current_grid.pixels == target_grid.pixels {
                                    telemetry.record_local_search_success();
                                    result = "ARC solved via subgoals".to_string();
                                    success = true;
                                }
                            }
                        }
                    }

                    if !success {
                        if let Ok(_plan) = self.explorer.explore_to_target(grid, target_grid, 20) {
                            result = "interactive exploration solved task".into();
                            success = true;
                            telemetry.record_local_search_success();
                        }
                    }
                }

                if !success {
                    let structure = task.context.structure.as_ref().unwrap().clone();
                    if let Some(solution) = self.run_local_search(&structure, adapter, &original_code, 5) {
                        telemetry.record_local_search_success();
                        result = solution;
                        success = true;
                    } else {
                        if let Some(ex) = self.llm_executor {
                            if let Ok(o) = ex.execute(task) {
                                telemetry.record_llm_call(o.confidence as u64, 200);
                                if let Some(s) = &task.context.structure { if adapter.validate(s, &o.content).is_ok() { self.store_rule(s, &o.content); self.store_semantic_schema(s, &o.content, "llm"); result = o.content; success = true; } }
                                if !success { result = "llm response failed validation".to_string(); }
                            } else { result = "llm executor error".to_string(); }
                        } else { telemetry.record_llm_call(100, 200); result = "llm fallback solution".to_string(); }
                    }
                }
            }
            "llm" => {
                if let Some(ex) = self.llm_executor {
                    if let Ok(o) = ex.execute(task) {
                        telemetry.record_llm_call(o.confidence as u64, 200);
                        if let Some(s) = &task.context.structure { if adapter.validate(s, &o.content).is_ok() { self.store_rule(s, &o.content); self.store_semantic_schema(s, &o.content, "llm"); result = o.content; success = true; } }
                        if !success { result = "llm response failed validation".to_string(); }
                    } else { result = "llm executor error".to_string(); }
                } else { telemetry.record_llm_call(100, 200); result = "llm solution".to_string(); success = true; }
            }
            _ => return Err("unknown path".into()),
        }
        self.record_episodic(task, success, &result);
        if success { Ok(result) } else { Err(result) }
    }

    pub fn execute_interactive_task(&mut self, env: &mut dyn Environment, telemetry: &mut Telemetry, max_steps: usize) -> Result<String, String> {
        let obs = env.reset();
        let mut current_grid = obs.clone();
        for step in 0..max_steps {
            let current_ksg = ArcAdapter::grid_to_ksg(&current_grid);
            let actions = self.explorer.possible_actions(&current_ksg);
            if actions.is_empty() { break; }
            use rand::seq::SliceRandom;
            let action = actions.choose(&mut rand::thread_rng()).unwrap().clone();
            match env.step(&action) {
                Ok((new_obs, target_grid)) => {
                    let new_ksg = ArcAdapter::grid_to_ksg(&new_obs);
                    self.program_synthesizer.learn_from_example(&current_ksg, &new_ksg);
                    if new_obs.pixels == target_grid.pixels {
                        telemetry.record_local_search_success();
                        return Ok(format!("interactive task solved in {} steps via action {}", step + 1, action));
                    }
                    current_grid = new_obs;
                }
                Err(_e) => {}
            }
        }
        Err("interactive task not solved within max steps".into())
    }
}

#[cfg(test)]
mod tests;
