use crate::structure::KernelStructureGraph;
use crate::memory::semantic::SemanticSchema;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Beolvassa a schemas.json fájlt és visszaadja a sémákat.
pub fn load_schemas(path: &Path) -> Result<HashMap<u64, SemanticSchema>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let entries: Vec<serde_json::Value> = serde_json::from_str(&content)?;
    let mut schemas = HashMap::new();
    
    for entry in entries {
        let original_code = entry["original_code"].as_str().unwrap_or("");
        let fixed_code = entry["fixed_code"].as_str().unwrap_or("");
        
        // Építsünk egy egyszerű gráfot a javításból
        let mut g = KernelStructureGraph::new();
        let node = g.add_node("fix1", "compiler_error");
        node.attributes.insert("action".into(), "fix_main".into());
        node.attributes.insert("solution".into(), fixed_code.to_string());
        
        let fp = g.fingerprint();
        let schema = SemanticSchema {
            id: uuid::Uuid::new_v4(),
            structure_snapshot: g,
            confidence: 0.5,
            domain_tags: vec!["compiler".into()],
        };
        schemas.insert(fp, schema);
    }
    Ok(schemas)
}
