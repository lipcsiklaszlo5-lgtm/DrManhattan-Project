#[cfg(test)]
mod tests {
    use crate::adapter::compiler::CompilerAdapter;
    use crate::adapter::DomainAdapter;
    use crate::task::{Task, TaskType};

    #[test]
    fn test_build_structure_e0308() {
        let task = Task::builder("fix error E0308").task_type(TaskType::CodeGeneration).build();
        let adapter = CompilerAdapter;
        let g = adapter.build_structure(&task);
        assert_eq!(g.nodes.len(), 1);
        assert_eq!(g.nodes[0].node_type, "compiler_error");
    }

    #[test]
    fn test_build_structure_lifetime() {
        let task = Task::builder("lifetime issue").task_type(TaskType::CodeGeneration).build();
        let adapter = CompilerAdapter;
        let g = adapter.build_structure(&task);
        assert_eq!(g.nodes.len(), 1);
        assert_eq!(g.nodes[0].node_type, "lifetime_error");
    }

    #[test]
    fn test_validate_pass() {
        let adapter = CompilerAdapter;
        let g = adapter.build_structure(&Task::builder("E0308").build());
        assert!(adapter.validate(&g, "this is the correct fix").is_ok());
    }

    #[test]
    fn test_validate_fail() {
        let adapter = CompilerAdapter;
        let g = adapter.build_structure(&Task::builder("E0308").build());
        assert!(adapter.validate(&g, "wrong answer").is_err());
    }

    #[test]
    fn test_available_algorithms() {
        let adapter = CompilerAdapter;
        let algs = adapter.available_algorithms();
        assert_eq!(algs.len(), 2);
        assert_eq!(algs[0].name, "cargo_fmt");
    }
}
