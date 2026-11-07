#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn test_fn_date_arguments() {
    let mut model = new_empty_model();

    model.set_a1("A1", "=DAY(95051806)");
    model.set_a1("A2", "=DAY(2958465)");
    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"#NUM!");
    assert_eq!(model.get_text_a1("A2"), *"31");
}
