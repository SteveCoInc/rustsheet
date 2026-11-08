#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn issue_623() {
    let mut model = new_empty_model();
    model.set_a1("A1", "=WORKDAY.INTL(46000, 5, 11, 46001)");
    model.set_a1("A2", "=WORKDAY.INTL(46000, 10, \"1111111\")"); // Weekend mask is all weekends

    model.evaluate();

    assert_eq!(model.get_text_a1("A1"), *"46007");
    assert_eq!(model.get_text_a1("A2"), *"#VALUE!");
}
