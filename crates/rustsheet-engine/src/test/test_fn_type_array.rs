#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// TYPE returns 64 for an array, as it does for a range.
#[test]
fn type_of_array_is_64() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=TYPE({1,2,3})");
    model.set_a1("A2", "=TYPE(SEQUENCE(3))");
    model.set_a1("A3", "=TYPE({1})");
    model.set_a1("A4", "=TYPE(B1:B3)");
    model.evaluate();
    assert_eq!(model.get_text_a1("A1"), "64");
    assert_eq!(model.get_text_a1("A2"), "64");
    assert_eq!(model.get_text_a1("A3"), "64");
    assert_eq!(model.get_text_a1("A4"), "64");
}
