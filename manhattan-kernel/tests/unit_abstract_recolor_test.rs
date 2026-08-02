use manhattan_kernel::abstraction::program::ProgramSynthesizer;
use manhattan_kernel::adapter::arc::adapter::{ArcGrid, ArcAdapter};

#[test]
fn test_abstract_recolor() {
    let mut synth = ProgramSynthesizer::new();
    let input = ArcGrid::new(2, 2, vec![1, 0, 0, 0]);
    let target = ArcGrid::new(2, 2, vec![2, 0, 0, 0]);

    let ksg_in = ArcAdapter::grid_to_ksg(&input);
    let ksg_out = ArcAdapter::grid_to_ksg(&target);

    // Tanulás
    synth.learn_from_example(&ksg_in, &ksg_out);
    assert!(!synth.programs.is_empty(), "Should have learned a program");
    
    // Ellenőrizzük, hogy a program tartalmaz Recolor-t
    let prog = &synth.programs[0];
    let has_recolor = prog.steps.iter().any(|s| matches!(s, manhattan_kernel::sandbox::operators::Transformation::Recolor { .. }));
    assert!(has_recolor, "Program should contain Recolor");
}
