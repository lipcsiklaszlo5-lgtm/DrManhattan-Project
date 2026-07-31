#[cfg(test)]
mod tests {
    use crate::adapter::compiler::CompilerAdapter;
    use crate::adapter::DomainAdapter;
    use crate::task::{Task, TaskType};
    use std::collections::HashMap;

    #[test]
    fn test_build_structure_from_real_error() {
        let code = "fn main() { let x: i32 = \"hello\"; }";
        let task = Task::builder(code).task_type(TaskType::CodeGeneration).build();
        let adapter = CompilerAdapter;
        let g = adapter.build_structure(&task);
        assert!(!g.nodes.is_empty());
        assert_eq!(g.nodes[0].node_type, "compiler_error");
        assert_eq!(g.nodes[0].attributes.get("action").unwrap(), "replace_type");
    }

    #[test]
    fn test_build_structure_no_error() {
        let code = "fn main() {}";
        let task = Task::builder(code).task_type(TaskType::CodeGeneration).build();
        let adapter = CompilerAdapter;
        let g = adapter.build_structure(&task);
        assert!(g.nodes.is_empty());
    }

    #[test]
    fn test_replace_type_generates_valid_code() {
        let mut g = crate::structure::KernelStructureGraph::new();
        let mut node = crate::structure::Node {
            id: "e1".into(),
            node_type: "compiler_error".into(),
            attributes: HashMap::new(),
        };
        node.attributes.insert("action".into(), "replace_type".into());
        node.attributes.insert("line".into(), "1".into());
        node.attributes.insert("column".into(), "18".into());
        node.attributes.insert("old_type".into(), "i32".into());
        node.attributes.insert("new_type".into(), "String".into());
        node.attributes.insert("new_value".into(), "\"hello\"".into());
        g.nodes.push(node);

        let adapter = CompilerAdapter;
        let code = "fn main() { let x: i32 = 5; }";
        let result = adapter.graph_to_code(&g, code);
        // Az eredmény: "fn main() { let x: String = \"hello\"; }"
        assert!(result.contains("String"));
        assert!(result.contains("\"hello\""));
        assert!(!result.contains("i32"));
    }

    #[test]
    fn test_add_import_generates_code() {
        let mut g = crate::structure::KernelStructureGraph::new();
        let mut node = crate::structure::Node {
            id: "e1".into(),
            node_type: "compiler_error".into(),
            attributes: HashMap::new(),
        };
        node.attributes.insert("action".into(), "add_import".into());
        node.attributes.insert("annotation".into(), "use std::io;".into());
        g.nodes.push(node);

        let adapter = CompilerAdapter;
        let code = "fn main() {}";
        let result = adapter.graph_to_code(&g, code);
        assert!(result.starts_with("use std::io;"));
    }

    #[test]
    fn test_rename_generates_code() {
        let mut g = crate::structure::KernelStructureGraph::new();
        let mut node = crate::structure::Node {
            id: "e1".into(),
            node_type: "compiler_error".into(),
            attributes: HashMap::new(),
        };
        node.attributes.insert("action".into(), "rename".into());
        node.attributes.insert("old_name".into(), "x".into());
        node.attributes.insert("new_name".into(), "corrected_var".into());
        g.nodes.push(node);

        let adapter = CompilerAdapter;
        let code = "fn main() { let x = 5; }";
        let result = adapter.graph_to_code(&g, code);
        assert!(result.contains("corrected_var"));
        assert!(!result.contains("let x"));
    }
}
