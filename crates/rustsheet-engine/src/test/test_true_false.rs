#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn true_false_arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TRUE( )");
    model.set_a1("A2", "=FALSE( )");
    model.set_a1("A3", "=TRUE( 4 )");
    model.set_a1("A4", "=FALSE( 4 )");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"TRUE");
    assert_eq!(model.get_text_a1("A2"), *"FALSE");

    // The run inside an empty call prints back (`=TRUE(` LF `)`; probes/t2966-empty-call-runs B3).
    assert_eq!(model.get_formula_a1("A1"), *"=TRUE( )");
    assert_eq!(model.get_formula_a1("A2"), *"=FALSE( )");

    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
    assert_eq!(model.get_formula_a1("A3"), *"=TRUE( 4 )");
    assert_eq!(model.get_formula_a1("A4"), *"=FALSE( 4 )");
}
