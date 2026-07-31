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
use crate::schema::composer::SchemaComposer;
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
}

impl<'a> PolicyEngine<'a> {
    pub fn new(cost_model: CostModel, candidate_gen: CandidateGenerator) -> Self {
        Self {
            cost_model, candidate_gen, rules: HashMap::new(), episodic_log: Vec::new(),
            semantic_schemas: HashMap::new(), schema_index: SchemaIndex::new(), llm_executor: None,
        }
    }

    pub fn with_llm_executor(mut self, executor: &'a LlmExecutor) -> Self { self.llm_executor = Some(executor); self }
    pub fn load_schemas(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let schemas = load_schemas(path)?;
        for (_fp, schema) in schemas { self.schema_index.insert(&schema); self.semantic_schemas.insert(schema.id, schema); }
        Ok(())
    }

    fn check_cache(&self, structure: &KernelStructureGraph) -> Option<&ProceduralRule> { self.rules.get(&structure.fingerprint()) }
    fn store_rule(&mut self, structure: &KernelStructureGraph, solution: &str) {
        let mut rule = ProceduralRule { id: uuid::Uuid::new_v4(), pattern: solution.to_string(), confidence: 0.5, success_count: 1, domain_tags: vec!["compiler".into()] };
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

    fn extract_predicates(&self, structure: &KernelStructureGraph) -> Vec<Predicate> {
        let mut preds = Vec::new();
        for node in &structure.nodes {
            if node.node_type == "arc_object" { preds.push(Predicate::TypeMismatch); }
        }
        preds
    }

    /// Kulcsfontosságú új metódus: transzformációs szabály kinyerése példákból
    pub fn learn_transformation(&mut self, before: &KernelStructureGraph, after: &KernelStructureGraph) -> Vec<NodeTransformation> {
        let diffs = graph_diff(before, after);
        // A sikeres transzformációkat eltároljuk sémaként
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
        if task.context.structure.is_some() {
            if let Some(s) = &task.context.structure {
                if s.nodes.is_empty() && s.edges.is_empty() { return "success"; }
                if self.check_cache(s).is_some() { return "cache"; }
            }
            return "schema_plan";
        }
        if self.cost_model.estimate_llm_cost(task) < 0.02 { return "llm"; }
        "llm"
    }

    pub fn run_local_search(&mut self, structure: &KernelStructureGraph, adapter: &dyn DomainAdapter, original_code: &str, max_candidates: usize) -> Option<String> {
        let action = structure.nodes.iter().find(|n| n.node_type == "compiler_error").and_then(|n| n.attributes.get("action").cloned()).unwrap_or_default();
        self.record_operator_attempt(&action);
        let candidates = self.candidate_gen.generate(structure, max_candidates);
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

    pub fn execute_task(&mut self, task: &mut Task, adapter: &dyn DomainAdapter, telemetry: &mut Telemetry) -> Result<String, String> {
        let original_code = task.intent.clone();
        if task.context.structure.is_none() { task.context.structure = Some(adapter.build_structure(task)); }
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
            "schema_plan" => {
                if let Some(structure) = &task.context.structure.clone() {
                    let predicates = self.extract_predicates(structure);
                    if !predicates.is_empty() {
                        let composer = SchemaComposer::new(self.schema_index.clone(), self.semantic_schemas.clone());
                        let plan = composer.compose(&predicates);
                        if !plan.is_empty() {
                            if let Some(step) = plan.first() {
                                if let Some(schema) = self.semantic_schemas.get(&step.schema_id) {
                                    let code = adapter.graph_to_code(&schema.graph, &original_code);
                                    if adapter.validate(structure, &code).is_ok() { telemetry.record_local_search_success(); self.store_rule(structure, &code); result = code; success = true; }
                                }
                            }
                        }
                    }
                }
                if !success {
                    let structure = task.context.structure.as_ref().unwrap().clone();
                    if let Some(solution) = self.run_local_search(&structure, adapter, &original_code, 5) { telemetry.record_local_search_success(); result = solution; success = true; }
                    else {
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
}

#[cfg(test)]
mod tests;
