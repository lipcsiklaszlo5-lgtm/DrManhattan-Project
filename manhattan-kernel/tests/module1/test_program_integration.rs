use manhattan_kernel::abstraction::program::{ProgramSynthesizer, GeneralizedProgram, AbstractStep, Cardinality};
use manhattan_kernel::sandbox::operators::Transformation;
use manhattan_kernel::adapter::arc::adapter::{ArcGrid, ArcAdapter};
use manhattan_kernel::predicate::builtin::ColorPredicate;
use manhattan_kernel::predicate::Predicate;

#[test]
fn test_generalized_program_with_predicate() {
    let mut synth = ProgramSynthesizer::new();
    let input = ArcGrid::new(3, 3, vec![1,0,0, 0,0,0, 0,0,0]);
    let target = ArcGrid::new(3, 3, vec![2,0,0, 0,0,0, 0,0,0]);
    let input_ksg = ArcAdapter::grid_to_ksg(&input);
    let target_ksg = ArcAdapter::grid_to_ksg(&target);

    // Tanulás generalizált programmal
    let gen_prog = synth.learn_generalized(&input_ksg, &target_ksg);
    assert!(gen_prog.is_some(), "Should learn a generalized program");

    // Alkalmazás a generalizált programmal
    if let Some(prog) = gen_prog {
        let result_ksg = prog.apply(&input_ksg, target.width, target.height);
        let result_grid = ArcAdapter::ksg_to_grid(&result_ksg, target.width, target.height, 0);
        assert_eq!(result_grid.pixels, target.pixels, "Generalized program should produce correct output");
    }
}
