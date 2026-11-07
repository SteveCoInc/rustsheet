#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=CEILING()");
    model.set_a1("A2", "=CEILING(5.2)");
    model.set_a1("A3", "=CEILING(5.2, 2)");
    model.set_a1("A4", "=CEILING(5.2, 2, 3)");

    model.set_a1("A5", "=CEILING.PRECISE()");
    model.set_a1("A6", "=CEILING.PRECISE(5.2)");
    model.set_a1("A7", "=CEILING.PRECISE(5.2, 2)");
    model.set_a1("A8", "=CEILING.PRECISE(5.2, 2, 3)");

    model.set_a1("A9", "=CEILING.MATH()");
    model.set_a1("A10", "=CEILING.MATH(5.2)");
    model.set_a1("A11", "=CEILING.MATH(5.2, 2)");
    model.set_a1("A12", "=CEILING.MATH(5.2, 2, 3)");
    model.set_a1("A13", "=CEILING.MATH(5.2, 2, 3, 4)");

    model.set_a1("B1", "=FLOOR()");
    model.set_a1("B2", "=FLOOR(5.2)");
    model.set_a1("B3", "=FLOOR(5.2, 2)");
    model.set_a1("B4", "=FLOOR(5.2, 2, 3)");

    model.set_a1("B5", "=FLOOR.PRECISE()");
    model.set_a1("B6", "=FLOOR.PRECISE(5.2)");
    model.set_a1("B7", "=FLOOR.PRECISE(5.2, 2)");
    model.set_a1("B8", "=FLOOR.PRECISE(5.2, 2, 3)");

    model.set_a1("B9", "=FLOOR.MATH()");
    model.set_a1("B10", "=FLOOR.MATH(5.2)");
    model.set_a1("B11", "=FLOOR.MATH(5.2, 2)");
    model.set_a1("B12", "=FLOOR.MATH(5.2, 2, 3)");
    model.set_a1("B13", "=FLOOR.MATH(5.2, 2, 3, 4)");

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A3"), *"6");
    assert_eq!(model.get_text_a1("A4"), *"#ERROR!");

    assert_eq!(model.get_text_a1("A5"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A6"), *"6");
    assert_eq!(model.get_text_a1("A7"), *"6");
    assert_eq!(model.get_text_a1("A8"), *"#ERROR!");

    assert_eq!(model.get_text_a1("A9"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A10"), *"6");
    assert_eq!(model.get_text_a1("A11"), *"6");
    assert_eq!(model.get_text_a1("A12"), *"6");
    assert_eq!(model.get_text_a1("A13"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B2"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B3"), *"4");
    assert_eq!(model.get_text_a1("B4"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B5"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B6"), *"5");
    assert_eq!(model.get_text_a1("B7"), *"4");
    assert_eq!(model.get_text_a1("B8"), *"#ERROR!");

    assert_eq!(model.get_text_a1("B9"), *"#ERROR!");
    assert_eq!(model.get_text_a1("B10"), *"5");
    assert_eq!(model.get_text_a1("B11"), *"4");
    assert_eq!(model.get_text_a1("B12"), *"4");
    assert_eq!(model.get_text_a1("B13"), *"#ERROR!");
}
