#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

#[test]
fn issue_155() {
    let mut model = new_empty_model();
    model.set_a1("A1", "1");
    model.set_a1("A2", "2");
    model.set_a1("B2", "=A$1:A2");
    model.evaluate();

    assert_eq!(model.get_formula_a1("B2"), "=A$1:A2".to_string());
}
