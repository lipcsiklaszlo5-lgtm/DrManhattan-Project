use manhattan_kernel::abstraction::program::{GeneralizedProgram, AbstractStep, Cardinality};
use manhattan_kernel::predicate::builtin::LargestPredicate;
use manhattan_kernel::sandbox::operators::Transformation;

fn is_semantically_pure(program: &GeneralizedProgram) -> bool {
    for step in &program.steps {
        // Check condition: no concrete node IDs in predicate names
        if let Some(cond) = &step.condition {
            let cond_name = cond.name();
            if cond_name.contains("obj_") || cond_name.contains("bbox_x") || cond_name.contains("bbox_y") {
                return false;
            }
        }
        // Check transformation
        match &step.transformation {
            Transformation::Translate { node_id, dx: _, dy: _ } => {
                if node_id.contains("obj_") || node_id.is_empty() {
                    return false;
                }
            }
            Transformation::MirrorHorizontal { node_id } | 
            Transformation::MirrorVertical { node_id } |
            Transformation::TranslateToTarget { node_id } |
            Transformation::RecolorToTarget { node_id } => {
                if node_id.contains("obj_") || node_id.is_empty() {
                    return false;
                }
            }
            Transformation::Create { bbox_x: _, bbox_y: _, .. } => {
                // absolute coordinates forbidden
                return false;
            }
            Transformation::Delete { node_id } => {
                if node_id.contains("obj_") {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
}

#[test]
fn test_empty_program_is_pure() {
    let program = GeneralizedProgram::new(vec![], 1.0, 0);
    assert!(is_semantically_pure(&program));
}

#[test]
fn test_program_with_semantic_condition_is_pure() {
    let step = AbstractStep {
        condition: Some(Box::new(LargestPredicate)),
        transformation: Transformation::TranslateToTarget { node_id: String::new() },
        target_spec: None,
        cardinality: Cardinality::All,
    };
    let program = GeneralizedProgram::new(vec![step], 1.0, 0);
    assert!(is_semantically_pure(&program));
}

#[test]
fn test_program_with_concrete_node_id_is_impure() {
    let step = AbstractStep {
        condition: None,
        transformation: Transformation::Translate { 
            node_id: "obj_7".to_string(), 
            dx: 5, 
            dy: -2 
        },
        target_spec: None,
        cardinality: Cardinality::All,
    };
    let program = GeneralizedProgram::new(vec![step], 1.0, 0);
    assert!(!is_semantically_pure(&program));
}

#[test]
fn test_program_with_create_is_impure() {
    let step = AbstractStep {
        condition: None,
        transformation: Transformation::Create {
            color: "4".to_string(),
            bbox_x: 7,
            bbox_y: 2,
            bbox_w: 1,
            bbox_h: 1,
        },
        target_spec: None,
        cardinality: Cardinality::All,
    };
    let program = GeneralizedProgram::new(vec![step], 1.0, 0);
    assert!(!is_semantically_pure(&program));
}
