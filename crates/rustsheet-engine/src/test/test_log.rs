#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=LOG(100)");
    model.set_a1("A2", "=LOG()");
    model.set_a1("A3", "=LOG(10000, 10)");
    model.set_a1("A4", "=LOG(100, 10, 1)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"2");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"4");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
}
