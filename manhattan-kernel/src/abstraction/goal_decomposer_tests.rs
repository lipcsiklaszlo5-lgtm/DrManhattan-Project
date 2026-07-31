#[cfg(test)]
mod tests {
    use crate::abstraction::goal_decomposer::GoalDecomposer;
    use crate::adapter::arc::adapter::ArcGrid;
    use crate::adapter::arc::ArcAdapter;

    #[test]
    fn test_decompose_recolor_and_translate() {
        let input = ArcGrid::new(3, 3, vec![1,1,0, 1,1,0, 0,0,0]);
        let target = ArcGrid::new(3, 3, vec![0,0,0, 0,2,2, 0,2,2]);
        let initial_ksg = ArcAdapter::grid_to_ksg(&input);
        let target_ksg = ArcAdapter::grid_to_ksg(&target);

        let subgoals = GoalDecomposer::decompose(&initial_ksg, &target_ksg);

        assert_eq!(subgoals.len(), 1, "Must find exactly 1 subgoal for single-object change");
        let sg = &subgoals[0];
        assert!(sg.description.contains("recolor"), "Subgoal must mention recolor");
        assert!(sg.description.contains("translate"), "Subgoal must mention translate");
        println!("GoalDecomposer correctly identified combined subgoal: {}", sg.description);
    }
}
