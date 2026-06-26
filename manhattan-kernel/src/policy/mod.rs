use crate::executor::Executor;
use std::collections::HashMap;
use crate::task::Task;
use crate::adapter::DomainAdapter;
use crate::candidate::CandidateGenerator;
use crate::structure::KernelStructureGraph;
use crate::telemetry::Telemetry;
use crate::memory::episodic::EpisodicEntry;
use crate::memory::semantic::SemanticSchema;
use crate::memory::procedural::ProceduralRule;
use crate::executor::LlmExecutor;

#[derive(Debug, Clone)]
pub struct CostModel {
    pub llm_cost_per_call: f32,
}

impl CostModel {
    pub fn estimate_llm_cost(&self, _task: &Task) -> f32 {
        self.llm_cost_per_call
    }
}

pub struct PolicyEngine<'a> {
    cost_model: CostModel,
    candidate_gen: CandidateGenerator,
    rules: HashMap<u64, ProceduralRule>,
    episodic_log: Vec<EpisodicEntry>,
    semantic_schemas: HashMap<u64, SemanticSchema>,
    llm_executor: Option<&'a LlmExecutor>,
}

impl<'a> PolicyEngine<'a> {
    pub fn new(cost_model: CostModel, candidate_gen: CandidateGenerator) -> Self {
        Self {
            cost_model,
            candidate_gen,
            rules: HashMap::new(),
            episodic_log: Vec::new(),
            semantic_schemas: HashMap::new(),
            llm_executor: None,
        }
    }

    pub fn with_llm_executor(mut self, executor: &'a LlmExecutor) -> Self {
        self.llm_executor = Some(executor);
        self
    }

    fn check_cache(&self, structure: &KernelStructureGraph) -> Option<&ProceduralRule> {
        let fp = structure.fingerprint();
        self.rules.get(&fp)
    }

    fn store_rule(&mut self, structure: &KernelStructureGraph, solution: &str) {
        let mut rule = ProceduralRule {
            id: uuid::Uuid::new_v4(),
            pattern: solution.to_string(),
            confidence: 0.5,
            success_count: 1,
            domain_tags: vec!["compiler".into()],
        };
        rule.record_success();
        self.rules.insert(structure.fingerprint(), rule);
    }

    fn record_operator_success(&mut self, action: &str) {
        let stats = self.candidate_gen.operator_stats.entry(action.to_string()).or_insert((0, 0));
        stats.0 += 1;
        stats.1 += 1;
    }

    fn record_operator_attempt(&mut self, action: &str) {
        let stats = self.candidate_gen.operator_stats.entry(action.to_string()).or_insert((0, 0));
        stats.1 += 1;
    }

    fn record_episodic(&mut self, task: &Task, success: bool, notes: &str) {
        let entry = EpisodicEntry {
            id: uuid::Uuid::new_v4(),
            task_intent: task.intent.clone(),
            success,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            notes: notes.to_string(),
        };
        self.episodic_log.push(entry);
    }

    fn store_semantic_schema(&mut self, structure: &KernelStructureGraph, _solution: &str, action: &str) {
        let fp = structure.fingerprint();
        let mut schema = SemanticSchema {
            id: uuid::Uuid::new_v4(),
            structure_snapshot: structure.clone(),
            confidence: 0.5,
            domain_tags: vec!["compiler".into()],
        };
        // Tároljuk a sikeres operátort is
        schema.structure_snapshot.nodes.iter_mut().for_each(|n| {
            if n.node_type == "compiler_error" {
                n.attributes.insert("successful_action".into(), action.to_string());
            }
        });
        self.semantic_schemas.insert(fp, schema);
    }

    pub fn decide(&self, task: &Task, _adapter: &dyn DomainAdapter) -> &str {
        if task.context.structure.is_some() {
            if let Some(structure) = &task.context.structure {
                if structure.nodes.is_empty() && structure.edges.is_empty() {
                    return "success";
                }
                if self.check_cache(structure).is_some() {
                    return "cache";
                }
            }
            return "local_search";
        }
        if self.cost_model.estimate_llm_cost(task) < 0.02 {
            return "llm";
        }
        "llm"
    }

    pub fn run_local_search(
        &mut self,
        structure: &KernelStructureGraph,
        adapter: &dyn DomainAdapter,
        original_code: &str,
        max_candidates: usize,
    ) -> Option<String> {
        let action = structure.nodes.iter()
            .find(|n| n.node_type == "compiler_error")
            .and_then(|n| n.attributes.get("action").cloned())
            .unwrap_or_default();

        self.record_operator_attempt(&action);

        let candidates = self.candidate_gen.generate(structure, max_candidates);
        for candidate_graph in &candidates {
            let code = adapter.graph_to_code(candidate_graph, original_code);
            if adapter.validate(structure, &code).is_ok() {
                self.store_rule(structure, &code);
                self.store_semantic_schema(structure, &code, &action);
                self.record_operator_success(&action);
                return Some(code);
            }
        }
        None
    }

    pub fn execute_task(
        &mut self,
        task: &mut Task,
        adapter: &dyn DomainAdapter,
        telemetry: &mut Telemetry,
    ) -> Result<String, String> {
        let original_code = task.intent.clone();

        if task.context.structure.is_none() {
            task.context.structure = Some(adapter.build_structure(task));
        }

        let path = self.decide(task, adapter);
        let mut result = String::new();
        let mut success = false;

        match path {
            "success" => {
                result = "already correct".to_string();
                success = true;
            }
            "cache" => {
                if let Some(structure) = &task.context.structure {
                    if let Some(rule) = self.check_cache(structure) {
                        telemetry.record_cache_hit();
                        result = rule.pattern.clone();
                        success = true;
                    }
                }
                if !success {
                    if let Some(executor) = self.llm_executor {
                        match executor.execute(task) {
                            Ok(output) => {
                                telemetry.record_llm_call(output.confidence as u64, 200);
                                if let Some(structure) = &task.context.structure {
                                    if adapter.validate(structure, &output.content).is_ok() {
                                        self.store_rule(structure, &output.content);
                                        self.store_semantic_schema(structure, &output.content, "llm");
                                        result = output.content;
                                        success = true;
                                    }
                                }
                                if !success {
                                    result = "llm response failed validation".to_string();
                                }
                            }
                            Err(_) => {
                                result = "llm executor error".to_string();
                            }
                        }
                    } else {
                        telemetry.record_llm_call(100, 200);
                        result = "llm fallback (cache miss, no executor)".to_string();
                    }
                }
            }
            "local_search" => {
                let structure = task.context.structure.as_ref().unwrap().clone();
                match self.run_local_search(&structure, adapter, &original_code, 5) {
                    Some(solution) => {
                        telemetry.record_local_search_success();
                        result = solution;
                        success = true;
                    }
                    None => {
                        if let Some(executor) = self.llm_executor {
                            match executor.execute(task) {
                                Ok(output) => {
                                    telemetry.record_llm_call(output.confidence as u64, 200);
                                    if let Some(structure) = &task.context.structure {
                                        if adapter.validate(structure, &output.content).is_ok() {
                                            self.store_rule(structure, &output.content);
                                            self.store_semantic_schema(structure, &output.content, "llm");
                                            result = output.content;
                                            success = true;
                                        }
                                    }
                                    if !success {
                                        result = "llm response failed validation".to_string();
                                    }
                                }
                                Err(_) => {
                                    result = "llm executor error".to_string();
                                }
                            }
                        } else {
                            telemetry.record_llm_call(100, 200);
                            result = "llm fallback solution".to_string();
                        }
                    }
                }
            }
            "llm" => {
                if let Some(executor) = self.llm_executor {
                    match executor.execute(task) {
                        Ok(output) => {
                            telemetry.record_llm_call(output.confidence as u64, 200);
                            if let Some(structure) = &task.context.structure {
                                if adapter.validate(structure, &output.content).is_ok() {
                                    self.store_rule(structure, &output.content);
                                    self.store_semantic_schema(structure, &output.content, "llm");
                                    result = output.content;
                                    success = true;
                                }
                            }
                            if !success {
                                result = "llm response failed validation".to_string();
                            }
                        }
                        Err(_) => {
                            result = "llm executor error".to_string();
                        }
                    }
                } else {
                    telemetry.record_llm_call(100, 200);
                    result = "llm solution".to_string();
                    success = true;
                }
            }
            _ => return Err("unknown path".into()),
        }

        self.record_episodic(task, success, &result);

        if success {
            Ok(result)
        } else {
            Err(result)
        }
    }
}

#[cfg(test)]
mod tests;
