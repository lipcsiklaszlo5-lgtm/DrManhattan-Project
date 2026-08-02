use manhattan_kernel::predicate::{Predicate, PredicateResult};
use manhattan_kernel::predicate::builtin::*;
use manhattan_kernel::structure::KernelStructureGraph;

#[test]
fn test_all_attribute_predicates() {
    let mut g = KernelStructureGraph::new();
    let n = g.add_node("a", "arc_object");
    n.attributes.insert("color".into(), "1".into());
    n.attributes.insert("area".into(), "5".into());
    n.attributes.insert("bbox_w".into(), "3".into());
    n.attributes.insert("bbox_h".into(), "4".into());
    n.attributes.insert("role".into(), "player".into());
    n.attributes.insert("shape_mask".into(), "0,0;1,0;1,1".into());

    let color = ColorPredicate { color: "1".into() };
    assert!(color.evaluate(&g).is_true());

    let area = AreaPredicate { min: Some(5), max: Some(5) };
    assert!(area.evaluate(&g).is_true());

    let width = WidthPredicate { width: 3 };
    assert!(width.evaluate(&g).is_true());

    let height = HeightPredicate { height: 4 };
    assert!(height.evaluate(&g).is_true());

    let role = RolePredicate { role: "player".into() };
    assert!(role.evaluate(&g).is_true());

    let shape = ShapePredicate { mask: "0,0;1,0;1,1".into() };
    assert!(shape.evaluate(&g).is_true());

    let pixel_count = PixelCountPredicate { count: 3 };
    assert!(pixel_count.evaluate(&g).is_true());

    let ratio = AspectRatioPredicate { ratio: 3.0 / 4.0 };
    assert!(ratio.evaluate(&g).is_true());
}
