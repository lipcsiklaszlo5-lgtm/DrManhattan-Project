#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use crate::task::{Constraint, ConstraintKind, Priority, Task, TaskId, TaskType};

    #[test]
    fn test_task_builder_basic() {
        let task = Task::builder("generate fibonacci")
            .task_type(TaskType::CodeGeneration)
            .build();
        assert_eq!(task.intent, "generate fibonacci");
        assert!(!task.is_subtask());
    }

    #[test]
    fn test_task_builder_full() {
        let parent_id = TaskId::new();
        let task = Task::builder("explain monads")
            .task_type(TaskType::Explanation)
            .priority(Priority::High)
            .constraint(Constraint { kind: ConstraintKind::MaxTokens(512) })
            .constraint(Constraint { kind: ConstraintKind::MaxLatencyMs(2000) })
            .working_dir(PathBuf::from("/workspace"))
            .file(PathBuf::from("src/main.rs"))
            .parent(parent_id.clone())
            .metadata("language", json!("rust"))
            .build();

        assert_eq!(task.intent, "explain monads");
        assert_eq!(task.task_type, TaskType::Explanation);
        assert_eq!(task.priority, Priority::High);
        assert_eq!(task.constraints.len(), 2);
        assert_eq!(task.constraints[0].kind, ConstraintKind::MaxTokens(512));
        assert_eq!(task.constraints[1].kind, ConstraintKind::MaxLatencyMs(2000));
        assert_eq!(task.context.working_dir, Some(PathBuf::from("/workspace")));
        assert_eq!(task.context.files, vec![PathBuf::from("src/main.rs")]);
        assert_eq!(task.parent_id, Some(parent_id));
        assert_eq!(task.metadata.get("language"), Some(&json!("rust")));
        assert!(task.is_subtask());
    }

    #[test]
    fn test_task_serialization() {
        let task = Task::builder("search for Rust async patterns")
            .task_type(TaskType::Search)
            .priority(Priority::Normal)
            .constraint(Constraint { kind: ConstraintKind::MaxTokens(1024) })
            .build();

        let serialized = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&serialized).unwrap();

        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.intent, deserialized.intent);
        assert_eq!(task.task_type, deserialized.task_type);
        assert_eq!(task.constraints.len(), deserialized.constraints.len());
        assert_eq!(task.priority, deserialized.priority);
        assert_eq!(task.created_at.timestamp_micros(), deserialized.created_at.timestamp_micros());
        assert_eq!(task.parent_id, deserialized.parent_id);
        assert_eq!(task.metadata, deserialized.metadata);
    }

    #[test]
    fn test_task_id_uniqueness() {
        let mut ids: HashMap<TaskId, bool> = HashMap::new();
        for _ in 0..100 {
            let id = TaskId::new();
            assert!(ids.insert(id.clone(), true).is_none());
        }
    }

    #[test]
    fn test_subtask_detection() {
        let parent = TaskId::new();
        let child = Task::builder("test").parent(parent).build();
        assert!(child.is_subtask());
    }

    #[test]
    fn test_constraint_variants() {
        let task = Task::builder("test")
            .constraint(Constraint { kind: ConstraintKind::RequiredExecutor("gpt-4".into()) })
            .constraint(Constraint { kind: ConstraintKind::ForbiddenExecutor("gpt-3".into()) })
            .constraint(Constraint { kind: ConstraintKind::MaxCostUsd(0.05) })
            .build();

        assert_eq!(task.constraints.len(), 3);
        assert!(task.constraints.iter().any(|c| c.kind == ConstraintKind::RequiredExecutor("gpt-4".into())));
        assert!(task.constraints.iter().any(|c| c.kind == ConstraintKind::MaxCostUsd(0.05)));
    }

    #[test]
    fn test_task_metadata() {
        let mut task = Task::builder("test").build();
        task.add_metadata("origin", json!("cli"));
        task.add_metadata("tags", json!(["urgent", "security"]));

        assert_eq!(task.metadata.get("origin"), Some(&json!("cli")));
        assert_eq!(task.metadata.get("tags"), Some(&json!(["urgent", "security"])));
    }
}
