#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn fn_degrees_radians_arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=DEGREES()");
    model.set_a1("A2", "=RADIANS()");
    model.set_a1("A3", "=RADIANS(180)");
    model.set_a1("A4", "=RADIANS(180, 2)");
    model.set_a1("A5", "=DEGREES(RADIANS(180))");
    model.set_a1("A6", "=DEGREES(1, 2)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"3.141592654");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A5"), *"180");
    assert_eq!(model.get_text_a1("A6"), *"#ERROR!");
}
