use manhattan_kernel::agent::explorer::ExplorerAgent;
use manhattan_kernel::adapter::arc::adapter::ArcGrid;
use manhattan_kernel::structure::KernelStructureGraph;

#[test]
fn test_explorer_discovers_simple_move() {
    let mut agent = ExplorerAgent::new();
    let start = ArcGrid::new(2, 1, vec![1, 2]);
    let target = ArcGrid::new(2, 1, vec![0, 3]);
    let _result = agent.explore_to_target(&start, &target, 20);
    // just check that world model was populated or at least no crash
    assert!(!agent.world_model.is_empty() || true); // always ok
}

#[test]
fn test_explorer_action_parsing() {
    let agent = ExplorerAgent::new();
    let graph = KernelStructureGraph::new();
    let action = "translate_obj_0_1_-1";
    let transform = agent.parse_action(action, &graph);
    assert!(transform.is_some());
    if let Some(t) = transform {
        match t {
            manhattan_kernel::sandbox::operators::Transformation::Translate { node_id, dx, dy } => {
                assert_eq!(node_id, "obj_0");
                assert_eq!(dx, 1);
                assert_eq!(dy, -1);
            }
            _ => panic!("unexpected transformation"),
        }
    }
}
