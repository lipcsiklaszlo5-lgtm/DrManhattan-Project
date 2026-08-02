use manhattan_kernel::object_selector::SelectionStrategy;

#[test]
fn test_strategy_variants_exist() {
    let strategies = vec![
        SelectionStrategy::Best,
        SelectionStrategy::TopK(3),
        SelectionStrategy::Threshold(0.5),
        SelectionStrategy::All,
        SelectionStrategy::Unique,
    ];
    assert_eq!(strategies.len(), 5);
}

#[test]
fn test_strategy_debug() {
    for s in &[
        SelectionStrategy::Best,
        SelectionStrategy::All,
        SelectionStrategy::Unique,
        SelectionStrategy::TopK(2),
        SelectionStrategy::Threshold(0.8),
    ] {
        let dbg = format!("{:?}", s);
        assert!(!dbg.is_empty());
    }
}
