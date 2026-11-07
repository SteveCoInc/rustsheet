#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_average_arguments() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=AVERAGE()");
    model.set_a1("A2", "=AVERAGEA()");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#ERROR!");
    assert_eq!(model.get_text_a1("A2"), *"#ERROR!");
}

#[test]
fn test_fn_average_minimal() {
    let mut model = new_empty_model();
    model.set_a1("B1", "1");
    model.set_a1("B2", "2");
    model.set_a1("B3", "3");
    model.set_a1("B4", "'2");
    // B5 is empty
    model.set_a1("B6", "true");
    model.set_a1("A1", "=AVERAGE(B1:B6)");
    model.set_a1("A2", "=AVERAGEA(B1:B6)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"2");
    assert_eq!(model.get_text_a1("A2"), *"1.4");
}
