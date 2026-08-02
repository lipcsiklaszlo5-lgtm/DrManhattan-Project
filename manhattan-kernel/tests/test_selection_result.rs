use manhattan_kernel::object_selector::SelectionResult;

#[test]
fn test_selection_result_default() {
    let result = SelectionResult {
        selected: vec![],
        ranking: vec![],
        ambiguity: false,
        confidence: 1.0,
        explanation: String::new(),
    };
    assert!(result.selected.is_empty());
    assert!(result.ranking.is_empty());
    assert!(!result.ambiguity);
    assert_eq!(result.confidence, 1.0);
}
