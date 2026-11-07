#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=LN(100)");
    model.set_a1("A2", "=LN()");
    model.set_a1("A3", "=LN(100, 10)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"4.605170186");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"#ERROR!");
}
