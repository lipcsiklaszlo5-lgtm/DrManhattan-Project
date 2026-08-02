use manhattan_kernel::abstraction::program::ProgramSynthesizer;
use manhattan_kernel::adapter::arc::adapter::{ArcGrid, ArcAdapter};

#[test]
fn test_abstract_recolor_invariance() {
    let mut synth = ProgramSynthesizer::new();
    let input1 = ArcGrid::new(2, 2, vec![1, 0, 0, 0]);
    let target1 = ArcGrid::new(2, 2, vec![2, 0, 0, 0]);
    let input2 = ArcGrid::new(2, 2, vec![3, 0, 0, 0]);
    let target2 = ArcGrid::new(2, 2, vec![4, 0, 0, 0]);

    let ksg1_in = ArcAdapter::grid_to_ksg(&input1);
    let ksg1_out = ArcAdapter::grid_to_ksg(&target1);
    let ksg2_in = ArcAdapter::grid_to_ksg(&input2);
    let ksg2_out = ArcAdapter::grid_to_ksg(&target2);

    // Tanulás két példából
    synth.learn_from_example(&ksg1_in, &ksg1_out);
    synth.learn_from_example(&ksg2_in, &ksg2_out);

    // Ellenőrizzük, hogy legalább egy program van
    assert!(!synth.programs.is_empty(), "Should have learned at least one program");
}
